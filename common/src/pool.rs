/// A simple pool for some resources
/// it returns immutable reference which is generated by maker
use async_trait::async_trait;
use futures::Stream;
use itertools::Itertools;
use parking_lot::Mutex;
use pin_project_lite::pin_project;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{ready, Context, Poll};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::PollSemaphore;

impl<M: MakeItem> Pool<M> {
    pub async fn with_cap(cap: usize, maker: M) -> Result<Self, M::Error> {
        let pool = Pool {
            cap,
            semaphore: PollSemaphore::new(Arc::new(Semaphore::new(cap))),
            maker,
            items: Arc::new(Mutex::new(Vec::default())),
        };
        pool.prepare(pool.cap).await?;
        Ok(pool)
    }

    async fn prepare(&self, size: usize) -> Result<(), M::Error> {
        for _ in 0..size {
            match self.maker.make_item().await {
                Ok(v) => {
                    // keep mutex guard has been released between await
                    let mut guard = self.items.lock();
                    guard.push(Entry::Idle(Arc::new(v)))
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
}

pin_project! {
    pub struct Pool<M: MakeItem> {
        cap: usize,
        #[pin]
        semaphore: PollSemaphore,
        maker: M,
        items: Arc<Mutex<Vec<Entry<M::Item>>>>,
    }
}

#[derive(Debug)]
enum Entry<V> {
    Idle(Arc<V>),
    Occupied(Arc<V>),
}

impl<V> Entry<V> {
    fn inner(&self) -> Arc<V> {
        match self {
            Entry::Idle(v) => v.clone(),
            Entry::Occupied(v) => v.clone(),
        }
    }
}

/// RAII wrapper for item
pub struct ItemGuard<M: MakeItem> {
    inner: Arc<M::Item>,
    // keep this around so that it is dropped when the item consumed
    _permit: Option<OwnedSemaphorePermit>,
    items: Arc<Mutex<Vec<Entry<M::Item>>>>,
    acquire_at: usize,
}

impl<M: MakeItem> Deref for ItemGuard<M> {
    type Target = M::Item;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<M: MakeItem> Drop for ItemGuard<M> {
    fn drop(&mut self) {
        let mut guard = self.items.lock();
        let before = guard.get(self.acquire_at).unwrap();
        guard[self.acquire_at] = Entry::Idle(before.inner());
    }
}

#[async_trait]
pub trait MakeItem {
    type Item;
    type Error;

    async fn make_item(&self) -> Result<Self::Item, Self::Error>;
}

impl<M: MakeItem> Stream for Pool<M>
where
    M::Item: Unpin,
{
    type Item = ItemGuard<M>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let permit = ready!(this.semaphore.as_mut().poll_acquire(cx));

        let mut guard = this.items.lock();

        let find = guard
            .iter()
            .find_position(|v| matches!(*v, Entry::Idle(_)))
            .map(|(pos, v)| (pos, v.inner()));

        if let Some((acquire_at, inner)) = find {
            guard[acquire_at] = Entry::Occupied(inner.clone());

            return Poll::Ready(Some(ItemGuard {
                inner,
                _permit: permit,
                items: this.items.clone(),
                acquire_at,
            }));
        }

        // unreachable
        Poll::Ready(None)
    }
}

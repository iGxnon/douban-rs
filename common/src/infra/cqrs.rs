// CQRS
// From https://github.com/KodrAus/rust-web-app#commands-and-queries

// Defined a CQRS basic model with trait `Command` and `Query`
// `Command` takes ownerships from types whereas `Query` borrows
// from types. The commands capture some domain interaction and work
// directly on entities whereas queries are totally arbitrary.
// The difference in receivers means commands can call queries
// but queries can't call commands.

use async_trait::async_trait;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

pub trait Args {
    type Output;
}

pub struct GetFrom<T, E, D> {
    from: D,
    _data: (PhantomData<T>, PhantomData<E>),
}

impl<T, E, D> GetFrom<T, E, D> {
    pub fn new(from: D) -> Self {
        Self {
            from,
            _data: (Default::default(), Default::default()),
        }
    }

    pub fn src(&self) -> &D {
        &self.from
    }
}

impl<T, E, D> Args for GetFrom<T, E, D> {
    type Output = Result<T, E>;
}

pub struct SetInto<E, D> {
    into: D,
    _data: PhantomData<E>,
}

impl<E, D> SetInto<E, D> {
    pub fn new(into: D) -> Self {
        Self {
            into,
            _data: Default::default(),
        }
    }

    pub fn dst(&self) -> &D {
        &self.into
    }
}

impl<E, D> Args for SetInto<E, D> {
    type Output = Result<(), E>;
}

#[async_trait]
pub trait Command<TArgs: Args> {
    async fn execute(self, input: TArgs) -> TArgs::Output;
}

#[async_trait]
pub trait Query<TArgs: Args> {
    async fn execute(&self, input: TArgs) -> TArgs::Output;
}

#[async_trait]
impl<TArgs, TCommand, TFuture> Command<TArgs> for TCommand
where
    TArgs: Args + Send + 'static,
    TCommand: (FnOnce(TArgs) -> TFuture) + Send,
    TFuture: Future<Output = TArgs::Output> + Send,
{
    async fn execute(self, input: TArgs) -> TArgs::Output {
        self(input).await
    }
}

#[async_trait]
impl<TArgs, TQuery, TFuture> Query<TArgs> for TQuery
where
    TArgs: Args + Send + 'static,
    TQuery: (Fn(TArgs) -> TFuture) + Sync,
    TFuture: Future<Output = TArgs::Output> + Send,
{
    async fn execute(&self, input: TArgs) -> TArgs::Output {
        self(input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn write_read_test() {
        struct AddValue(i32);
        struct GetLen;

        impl Args for GetLen {
            type Output = usize;
        }

        impl Args for AddValue {
            type Output = ();
        }

        let mut data = vec![];

        fn add_value(data: &mut Vec<i32>) -> impl Command<AddValue> + '_ {
            move |value: AddValue| async move {
                data.push(value.0);
            }
        }

        fn get_len(data: &[i32]) -> impl Query<GetLen> + '_ {
            move |_| async move { data.len() }
        }

        let write = add_value(&mut data);

        // panic because it borrowed as mutable before
        // let query = get_len(&data);

        write.execute(AddValue(1)).await;
        assert_eq!(data, vec![1]);

        let read = get_len(&data);
        assert_eq!(1, read.execute(GetLen).await);
        assert_eq!(1, read.execute(GetLen).await);
    }
}

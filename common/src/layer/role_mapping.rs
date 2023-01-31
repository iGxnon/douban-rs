/// Casbin role mapping layer
///
use casbin::CoreApi;
use futures::future::BoxFuture;
use http::{Request, Response, StatusCode};
use std::marker::PhantomData;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::error;

#[derive(Clone)]
pub struct RoleMappingLayer<I, E> {
    enforcer: Arc<E>,
    _data: PhantomData<I>,
}

impl<I, E: CoreApi> RoleMappingLayer<I, E> {
    pub fn new_static(enforcer: E) -> Self {
        Self {
            enforcer: Arc::new(enforcer),
            _data: PhantomData::default(),
        }
    }

    pub fn from_arc(enforcer: Arc<E>) -> Self {
        Self {
            enforcer,
            _data: PhantomData::default(),
        }
    }
}

impl<S, I, E> Layer<S> for RoleMappingLayer<I, E> {
    type Service = RoleMapping<S, I, E>;

    fn layer(&self, inner: S) -> Self::Service {
        RoleMapping {
            inner,
            enforcer: self.enforcer.clone(),
            _data: PhantomData::default(),
        }
    }
}

#[derive(Clone)]
pub struct RoleMapping<S, I, E> {
    inner: S,
    enforcer: Arc<E>,
    _data: PhantomData<I>,
}

impl<S, I, E, ReqBody, ResBody> Service<Request<ReqBody>> for RoleMapping<S, I, E>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Send + 'static,
    S::Future: Send + 'static,
    ResBody: Default,
    I: AsRef<str> + Send + Sync + 'static,
    E: CoreApi,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let sub = req
            .extensions()
            .get::<I>()
            .map(|sub| sub.as_ref())
            .unwrap_or("");
        let obj = req.uri().path();
        let act = req.method().as_str();

        match self.enforcer.enforce((sub, obj, act)) {
            Ok(checked) => {
                if checked {
                    let fut = self.inner.call(req);
                    Box::pin(async move { fut.await })
                } else {
                    Box::pin(async move {
                        Ok(Response::builder()
                            .status(StatusCode::FORBIDDEN)
                            .body(ResBody::default())
                            .unwrap())
                    })
                }
            }
            Err(err) => {
                error!("enforcer is working abnormally, err: {:?}", err);
                Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(ResBody::default())
                        .unwrap())
                })
            }
        }
    }
}

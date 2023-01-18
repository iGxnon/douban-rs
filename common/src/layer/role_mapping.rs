// Casbin 访问控制中间件
// 必须在 HttpAuth 层后面 以获取从 HttpAuth 层写入的 sub

use casbin::{CoreApi, Enforcer};
use futures::future::BoxFuture;
use http::{Request, Response, StatusCode};
use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{event, Level};

#[derive(Clone)]
pub struct RoleMappingLayer<U> {
    enforcer: Arc<Enforcer>,
    _data: PhantomData<U>,
}

impl<U> RoleMappingLayer<U> {
    pub fn new(enforcer: Enforcer) -> Self {
        Self {
            enforcer: Arc::new(enforcer),
            _data: PhantomData::default(),
        }
    }
}

impl<S, U> Layer<S> for RoleMappingLayer<U> {
    type Service = RoleMapping<S, U>;

    fn layer(&self, inner: S) -> Self::Service {
        RoleMapping {
            inner,
            enforcer: self.enforcer.clone(),
            _data: PhantomData::default(),
        }
    }
}

#[derive(Clone)]
pub struct RoleMapping<S, U> {
    inner: S,
    enforcer: Arc<Enforcer>,
    _data: PhantomData<U>,
}

impl<S, ReqBody, ResBody, U> Service<Request<ReqBody>> for RoleMapping<S, U>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Send + 'static,
    S::Future: Send + 'static,
    ResBody: Default,
    U: Display + Send + Sync + 'static,
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
            .get::<U>()
            .map(ToString::to_string)
            .unwrap_or_else(|| "".to_string());
        let obj = req.uri().path();
        let act = req.method().as_str();
        event!(
            Level::TRACE,
            target = "layer:role_map",
            "start to enforce sub({}), obj({}), act({})",
            sub,
            obj,
            act
        );
        match self.enforcer.enforce((&*sub, obj, act)) {
            Ok(checked) => {
                if checked {
                    let fut = self.inner.call(req);
                    Box::pin(async move { fut.await })
                } else {
                    event!(
                        Level::INFO,
                        target = "layer:role_map",
                        "enforce sub({}), obj({}), act({}) failed, no authorized!",
                        sub,
                        obj,
                        act
                    );
                    Box::pin(async move {
                        Ok(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body(ResBody::default())
                            .unwrap())
                    })
                }
            }
            Err(err) => {
                event!(
                    Level::ERROR,
                    target = "layer:role_map",
                    "enforcer is working abnormally, err: {:?}",
                    err
                );
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

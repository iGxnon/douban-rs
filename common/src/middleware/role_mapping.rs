// Casbin 访问控制中间件
// 必须在 HttpAuth 层后面

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
    pub fn new(enforcer: Arc<Enforcer>) -> Self {
        Self {
            enforcer,
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
            target = "middleware:role_map",
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
                        target = "middleware:role_map",
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
                    target = "middleware:role_map",
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

mod test {
    use crate::consts::AUTH_HEADER;
    use crate::middleware::role_mapping::RoleMappingLayer;
    use crate::middleware::{AuthBackend, HttpAuthLayer};
    use crate::model::UserId;
    use crate::model::UserId::UserIdU64;
    use casbin::{CoreApi, Enforcer};
    use futures::future::BoxFuture;
    use http::{header, Method, Request, Response, StatusCode};
    use hyper::Body;
    use std::sync::Arc;
    use tower::{ServiceBuilder, ServiceExt};
    use tower_http::cors::CorsLayer;

    #[derive(Copy, Clone, Debug)]
    struct BlindBackend;

    impl AuthBackend<UserId> for BlindBackend {
        fn auth_basic(&self, _: &str, _: &str) -> Result<UserId, String> {
            Ok(UserIdU64(0))
        }

        fn auth_basic_async(
            &self,
            _: &str,
            _: &str,
        ) -> BoxFuture<'static, Result<(UserId, Option<String>), String>> {
            Box::pin(async { Ok((UserIdU64(0), None)) })
        }

        fn auth_bearer(
            &self,
            _: &str,
        ) -> BoxFuture<'static, Result<(UserId, Option<String>), String>> {
            Box::pin(async { Ok((UserIdU64(0), None)) })
        }
    }

    #[tokio::test]
    async fn test_role_mapping() {
        tracing_subscriber::fmt::init();

        let enforcer = Enforcer::new(
            "../resources/rolemap/casbin_model.conf",
            "../resources/rolemap/test_policy.csv",
        )
        .await
        .unwrap();

        let app = ServiceBuilder::new()
            .boxed()
            .layer(HttpAuthLayer::without_cookie(BlindBackend, false))
            .layer(RoleMappingLayer::<UserId>::new(Arc::new(enforcer)))
            .layer(CorsLayer::very_permissive().allow_credentials(true))
            .service_fn(|_req: Request<()>| async {
                Result::<Response<Body>, String>::Ok(Response::new(Body::empty()))
            });
        let a = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("https://api.douban.skygard.com/foo?start=5")
                    .header(AUTH_HEADER, "Bearer xxx")
                    .header(header::ORIGIN, "https://douban.skygard.com")
                    .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "X-Auth")
                    .body(())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(a.status(), StatusCode::OK);
    }
}

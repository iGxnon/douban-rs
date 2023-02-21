use crate::auth::layer::{AuthBuilder, WWWAuth};
use crate::user::rest::handler::bind;
use crate::user::rest::handler::login;
use crate::user::rest::handler::register;
use crate::user::rest::types::IdProvider;
use crate::user::rest::RestResolver;
use axum::routing::post;
use axum::Router;
use common::layer::AsyncHttpAuthLayer;
use std::sync::Arc;
use tower::ServiceBuilder;
use common::infra::Resolver;

impl RestResolver {
    pub async fn make_router(&self) -> Router {
        let auth = AuthBuilder::cookie(self.conf.etcd.clone())
            .cookie_conf(self.conf.cookie_conf.clone())
            .www(WWWAuth::cookie(Self::DOMAIN, self.conf.cookie_conf.cookie_name.as_str()))
            .finish::<IdProvider, _>()
            .await;
        let bind_router = Router::new()
            .route("/bind", post(bind::handle))
            .layer(ServiceBuilder::new().layer(AsyncHttpAuthLayer::new(auth)));
        Router::new()
            .route("/register", post(register::handle))
            .route("/login", post(login::handle))
            .merge(bind_router)
            .with_state(Arc::new(self.clone()))
    }
}

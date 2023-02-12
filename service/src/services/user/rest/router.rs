use crate::auth::layer::{Auth, AuthConf, IdentityProvider};
use crate::user::rest::handler::bind;
use crate::user::rest::handler::login;
use crate::user::rest::handler::register;
use crate::user::rest::RestResolver;
use axum::routing::post;
use axum::Router;
use common::layer::AsyncHttpAuthLayer;
use std::sync::Arc;
use tower::ServiceBuilder;

#[derive(Clone)]
struct IdProvider;

impl IdentityProvider for IdProvider {
    type Id = String;
    type Group = String;
    type Extra = String;
}

impl RestResolver {
    pub async fn make_router(&self) -> Router {
        let auth = Auth::<IdProvider, _>::cookie(AuthConf::default()).await;
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

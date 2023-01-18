use crate::api::handler::bind;
use crate::api::handler::login;
use crate::api::handler::register;
use crate::api::{error, Resolver};
use axum::error_handling::HandleErrorLayer;
use axum::routing::post;
use axum::Router;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

pub fn router(resolver: Resolver) -> Router {
    let bind_router = Router::new().route("/bind", post(bind::handle));
    Router::new()
        .route("/register", post(register::handle))
        .route("/login", post(login::handle))
        .merge(bind_router)
        .layer(
            ServiceBuilder::new()
                .catch_panic()
                .trace_for_http()
                .layer(HandleErrorLayer::new(error::handle_error))
                .timeout(Duration::from_secs(1))
                .load_shed()
                .concurrency_limit(512),
        )
        .with_state(Arc::new(resolver))
}

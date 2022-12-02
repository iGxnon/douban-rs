use auth::Backend;
use axum::{routing::any, Router, Server};
use casbin::{CoreApi, Enforcer};
use common::middleware::{HttpAuthLayer, RoleMappingLayer};
use common::model::UserId;
use http::StatusCode;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let enforcer = Enforcer::new(
        "resources/rolemap/casbin_model.conf",
        "resources/rolemap/test_policy.csv",
    )
    .await
    .unwrap();

    let app = Router::new()
        .route("/foo", any(|| async { (StatusCode::OK, "hi") }))
        .layer(RoleMappingLayer::<UserId>::new(Arc::new(enforcer)))
        .layer(HttpAuthLayer::new(Backend, false, false, 172800));

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

use axum::Router;
use common::config::env::require;
use common::utils::parse_config;
use service::user::rest::{RestConfig, RestResolver};

/// douban-rs Backend for frontend
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let conf: RestConfig = parse_config::<RestResolver>()
        .await
        .expect("Cannot parse config");
    let rest_resolver = RestResolver::new(conf).await;

    let route = rest_resolver.make_router().await;

    serve(&require("APP_ADDR"), Router::new().merge(route)).await
}

pub async fn serve(listen_addr: &str, route: Router) {
    let addr = listen_addr.parse().unwrap();
    axum::Server::bind(&addr)
        .serve(route.into_make_service())
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
}

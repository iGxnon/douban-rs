use axum::Router;
use common::config::env::require;
use common::utils::parse_config;
use futures::FutureExt;
use service::user::rest::{RestConfig as UserConfig, RestResolver as UserResolver};

/// douban-rs BFF(Backend for Frontend), merged all routers into one service
/// and provide only one endpoint to frontend
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut root_router = Router::new();

    {
        // user router
        let conf: UserConfig = parse_config::<UserResolver>()
            .await
            .expect("Cannot parse user config");
        let route = UserResolver::new(conf).await.make_router().await;
        root_router = root_router.merge(route)
    }

    serve(&require("APP_ADDR"), root_router).await
}

pub async fn serve(listen_addr: &str, route: Router) {
    let addr = listen_addr.parse().unwrap();
    axum::Server::bind(&addr)
        .serve(route.into_make_service())
        .with_graceful_shutdown(tokio::signal::ctrl_c().map(|_| ()))
        .await
        .unwrap();
}

use user::api::route::router;
use user::api::{Config, Resolver};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router(Resolver::new(Config {})).into_make_service())
        .await
        .unwrap();
}

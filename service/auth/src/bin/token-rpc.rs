use auth::domain::token::{Resolver, TokenConfig};
use common::discover::Discover;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = TokenConfig::default();
    println!("{:?}", config);

    let resolver = Resolver::new(config);

    let discover = resolver.make_discover();
    discover
        .register_service()
        .await
        .expect("Cannot register service into etcd");

    let serve = resolver.make_serve().await;
    serve.await.unwrap();
}

use auth::domain::token::{TokenConfig, TokenResolver};
use common::discover::Discover;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = TokenConfig::default();

    println!("{}", serde_json::to_string_pretty(&config).unwrap());

    let mut resolver = TokenResolver::new(config);

    resolver.add_expire("auth", 167800);

    let discover = resolver.make_discover();
    discover
        .register_service(&resolver)
        .await
        .expect("Cannot register service into etcd");

    let serve = resolver.make_serve().await;
    serve.await.unwrap();
}

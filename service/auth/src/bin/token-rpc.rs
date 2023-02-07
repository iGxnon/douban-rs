use auth::domain::token::TokenResolver;
use common::infra::Resolver;
use common::utils::parse_config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = parse_config(TokenResolver::DOMAIN)
        .await
        .expect("Cannot parse config");

    println!("{}", serde_json::to_string_pretty(&config).unwrap());

    let resolver = TokenResolver::new(config);

    resolver.register_service().await;

    resolver.serve().await.expect("Start failed");
}

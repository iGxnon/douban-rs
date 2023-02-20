use common::infra::Resolver;
use common::utils::{config_tips, parse_config};
use service::auth::domain::token::TokenResolver;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = parse_config(TokenResolver::DOMAIN)
        .await
        .expect("Cannot parse config");

    config_tips(&config);

    let resolver = TokenResolver::new(config);

    resolver.register_service().await;

    resolver.serve().await.expect("Start failed");
}

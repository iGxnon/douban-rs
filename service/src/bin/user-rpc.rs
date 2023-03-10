use common::utils::{config_tips, parse_config};
use service::user::rpc::UserResolver;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = parse_config::<UserResolver>()
        .await
        .expect("Cannot parse config");

    config_tips(&config);

    let resolver = UserResolver::new(config).await;

    resolver.register_service().await;

    resolver.serve().await.expect("Start failed");
}

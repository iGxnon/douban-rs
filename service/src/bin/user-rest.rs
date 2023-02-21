use common::utils::{config_tips, parse_config};
use service::user::rest::{RestConfig, RestResolver};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let conf: RestConfig = parse_config::<RestResolver>()
        .await
        .expect("Cannot parse config");

    config_tips(&conf);

    let resolver = RestResolver::new(conf).await;

    resolver.serve().await;
}

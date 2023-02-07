use common::infra::Resolver;
use common::utils::parse_config;
use user::rest::RestResolver;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let conf = parse_config(RestResolver::DOMAIN)
        .await
        .expect("Cannot parse config");

    println!("{}", serde_json::to_string_pretty(&conf).unwrap());

    let resolver = RestResolver::new(conf).await;

    resolver.serve().await;
}

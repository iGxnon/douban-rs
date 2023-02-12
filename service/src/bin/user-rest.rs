use common::infra::Resolver;
use common::utils::parse_config;
use service::user::rest::{RestConfig, RestResolver};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut conf: RestConfig = parse_config(RestResolver::DOMAIN)
        .await
        .expect("Cannot parse config");

    conf.service_conf.service.listen_addr = "0.0.0.0:5001".to_string();

    println!("{}", serde_json::to_string_pretty(&conf).unwrap());

    let resolver = RestResolver::new(conf).await;

    resolver.serve().await;
}

use auth::domain::token::{Config, Resolver, SERVICE_NAME};
use auth::rpc::token::TokenService;
use auth::token_srv_server::TokenSrvServer;
use common::discover::register_service;
use std::time::Duration;
use tonic::transport::Server;
use tracing::{info, info_span};

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let config = common::utils::parse_config::<Config>(SERVICE_NAME).await?;

    let health_check = config.health_check;
    let timeout = config.timeout;
    let concurrency_limit = config.concurrency_limit;
    let addr = config.listen_addr.parse()?;
    let endpoint = config.endpoint.clone();
    let etcd = config.etcd_clients.clone();
    let grant_ttl = config.etcd_grant_ttl;
    let keepalive_interval = config.etcd_keepalive_interval;

    // create service
    let service = TokenService::new(Resolver::new(config)).into_server();

    info!("TokenServer listen on {}", addr);

    // register into etcd
    let mut etcd_client = etcd_client::Client::connect(etcd, None).await?;

    register_service(
        &mut etcd_client,
        SERVICE_NAME,
        &endpoint,
        grant_ttl,
        keepalive_interval,
    )
    .await?;

    info!("TokenServer registered {} into etcd", endpoint);

    if health_check {
        let (mut reporter, health_service) = tonic_health::server::health_reporter();
        reporter.set_serving::<TokenSrvServer<TokenService>>().await;

        info!("Enabled GRPC health check");

        Server::builder()
            .trace_fn(|_| info_span!(SERVICE_NAME))
            .timeout(Duration::from_secs(timeout))
            .concurrency_limit_per_connection(concurrency_limit)
            .add_service(service)
            .add_service(health_service)
            .serve(addr)
            .await?;
    } else {
        Server::builder()
            .trace_fn(|_| info_span!(SERVICE_NAME))
            .timeout(Duration::from_secs(timeout))
            .concurrency_limit_per_connection(concurrency_limit)
            .add_service(service)
            .serve(addr)
            .await?;
    }

    Ok(())
}

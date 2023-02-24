use crate::user::rest::error::handle_error;
use crate::user::rpc::UserResolver;
use axum::error_handling::HandleErrorLayer;
use common::config::layer::LayerConfig;
use common::config::middleware::MiddlewareConfig;
use common::config::service::ServiceConfig;
use common::config::Config;
use common::infra::*;
use common::registry::{EtcdRegistry, ServiceDiscover};
use proto::pb::user::sys::v1::user_service_client::UserServiceClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tonic::transport::Channel;
use tower::load_shed::LoadShedLayer;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

mod error;
mod handler;
mod router;
mod types;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RestConfig {
    #[serde(default)]
    pub service_conf: <Config as ServiceConfig>::RestService,
    #[serde(default)]
    pub etcd: <Config as MiddlewareConfig>::Etcd,
    #[serde(default)]
    pub cookie_conf: <Config as LayerConfig>::CookieAuth,
}

#[derive(Clone)]
pub struct RestResolver {
    conf: RestConfig,
    user_client: UserServiceClient<Channel>,
}

impl Resolver for RestResolver {
    const TARGET: Target = Target::REST;
    const DOMAIN: &'static str = "user";
    type Config = RestConfig;

    fn conf(&self) -> &Self::Config {
        &self.conf
    }
}

impl RestResolver {
    pub async fn new(conf: RestConfig) -> Self {
        let registry = EtcdRegistry::discover(conf.etcd.clone());
        let (channel, tx) = Channel::balance_channel(1024);
        let service_key = UserResolver::service_key();
        registry
            .discover_to_channel(&service_key, tx)
            .await
            .expect("Cannot discover user service to channel");
        let user_client = UserServiceClient::new(channel);
        Self { conf, user_client }
    }

    pub fn user_client(&self) -> UserServiceClient<Channel> {
        self.user_client.clone()
    }

    pub async fn serve(&self) {
        let addr = self.conf.service_conf.service.listen_addr.parse().unwrap();
        axum::Server::bind(&addr)
            .serve(
                self.make_router()
                    .await
                    .layer(
                        ServiceBuilder::new()
                            .catch_panic()
                            .trace_for_http()
                            .layer(HandleErrorLayer::new(handle_error))
                            .timeout(Duration::from_secs(self.conf.service_conf.service.timeout))
                            .option_layer(if self.conf.service_conf.service.load_shed {
                                Some(LoadShedLayer::new())
                            } else {
                                None
                            })
                            .concurrency_limit(self.conf.service_conf.service.concurrency_limit),
                    )
                    .into_make_service(),
            )
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await
            .unwrap();
    }
}

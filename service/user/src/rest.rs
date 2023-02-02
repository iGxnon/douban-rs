use crate::rest::error::handle_error;
use axum::error_handling::HandleErrorLayer;
use common::config::register;
use common::config::service::ServiceConfig;
use common::config::Config;
use common::infra::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tower::load_shed::LoadShedLayer;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

mod error;
mod handler;
mod router;
mod types;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct RestConfig {
    pub service_conf: <Config as ServiceConfig>::ApiService,
}

type Register<T> = register::Register<RestConfig, T>;

#[derive(Debug, Clone)]
pub struct RestResolver {
    conf: RestConfig,
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
    pub fn new(conf: RestConfig) -> Self {
        Self { conf }
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
            .await
            .unwrap();
    }
}

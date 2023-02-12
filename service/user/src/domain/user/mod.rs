pub mod command;
pub mod model;
pub mod query;

use crate::rpc::user::UserService;
use common::config::middleware::MiddlewareConfig;
use common::config::service::ServiceConfig;
use common::config::Config;
use common::infra::{Resolver, Target};
use common::registry::{EtcdRegistry, ServiceRegister};
use proto::pb::user::sys::v1::user_service_server::UserServiceServer;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tonic::transport::Server;
use tower::load_shed::LoadShedLayer;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    #[serde(default)]
    service_conf: <Config as ServiceConfig>::GrpcService,
    #[serde(default)]
    redis: <Config as MiddlewareConfig>::Redis,
    #[serde(default)]
    etcd: <Config as MiddlewareConfig>::Etcd,
}

type Register<T> = common::config::register::Register<UserConfig, T>;

#[derive(Clone)]
pub struct UserResolver {
    conf: UserConfig,
}

impl Resolver for UserResolver {
    const TARGET: Target = Target::GRPC;

    const DOMAIN: &'static str = "user";

    type Config = UserConfig;

    fn conf(&self) -> &Self::Config {
        &self.conf
    }
}

impl UserResolver {
    pub fn new(conf: UserConfig) -> Self {
        Self { conf }
    }

    pub async fn register_service(&self) {
        let registry = EtcdRegistry::register(
            self.conf.etcd.clone(),
            self.conf.service_conf.service.clone(),
        );
        registry
            .register_service(Self::DOMAIN)
            .await
            .expect("Cannot register service into etcd");
    }

    pub async fn serve(&self) -> Result<(), tonic::transport::Error> {
        let token_srv = UserService(self.clone());
        let addr = self
            .conf
            .service_conf
            .service
            .listen_addr
            .parse()
            .expect("cannot parse a valid listen_addr");

        let layer = ServiceBuilder::new()
            .catch_panic()
            .trace_for_grpc()
            .option_layer(if self.conf.service_conf.service.load_shed {
                Some(LoadShedLayer::new())
            } else {
                None
            })
            .concurrency_limit(self.conf.service_conf.service.concurrency_limit);

        let serve = Server::builder()
            .timeout(Duration::from_secs(self.conf.service_conf.service.timeout))
            .layer(layer)
            .add_optional_service(if self.conf.service_conf.health_check {
                let (mut reporter, svc) = tonic_health::server::health_reporter();
                reporter
                    .set_serving::<UserServiceServer<UserService>>()
                    .await;
                Some(svc)
            } else {
                None
            })
            .add_service(UserServiceServer::new(token_srv));

        serve.serve(addr).await
    }
}

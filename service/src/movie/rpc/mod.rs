use common::config::middleware::MiddlewareConfig;
use common::config::service::ServiceConfig;
use common::config::Config;
use common::infra::{Resolver, Target};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovieConfig {
    #[serde(default)]
    service_conf: <Config as ServiceConfig>::GrpcService,
    #[serde(default)]
    redis: <Config as MiddlewareConfig>::Redis,
    #[serde(default)]
    etcd: <Config as MiddlewareConfig>::Etcd,
}

#[derive(Clone)]
pub struct MovieResolver {}

impl Resolver for MovieResolver {
    const TARGET: Target = Target::GRPC;
    const DOMAIN: &'static str = "movie";
    type Config = ();

    fn conf(&self) -> &Self::Config {
        todo!()
    }
}

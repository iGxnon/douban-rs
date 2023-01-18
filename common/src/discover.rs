pub mod etcd;

use crate::config::service::ServiceConf;
use crate::middleware::consul::ConsulConf;
use crate::middleware::etcd::EtcdConf;
use async_trait::async_trait;
use std::hash::Hash;

pub use etcd::*;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use tonic::transport::Endpoint;
use tower::discover::Change;

#[async_trait]
pub trait Discover<K>
where
    K: Hash + Eq + Send + Clone + 'static,
{
    type Error;

    async fn register_service(&self) -> Result<(), Self::Error>;

    async fn discover_to_channel(&self, tx: Sender<Change<K, Endpoint>>)
        -> Result<(), Self::Error>;
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct EtcdDiscoverConf {
    pub etcd: EtcdConf,
    pub service: ServiceConf,
    pub grant_ttl: i64,
    pub keep_alive_interval: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default)]
pub struct ConsulDiscoverConf {
    pub consul: ConsulConf,
    pub service: ServiceConf,
}

impl EtcdDiscoverConf {
    pub fn new(etcd: EtcdConf, service: ServiceConf) -> Self {
        Self {
            etcd,
            service,
            grant_ttl: 61,
            keep_alive_interval: 20,
        }
    }
}

impl Default for EtcdDiscoverConf {
    fn default() -> Self {
        Self {
            etcd: Default::default(),
            service: Default::default(),
            grant_ttl: 61,
            keep_alive_interval: 20,
        }
    }
}

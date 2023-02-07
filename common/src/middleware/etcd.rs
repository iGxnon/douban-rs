use crate::config::env::optional;
use crate::middleware::Middleware;
use async_trait::async_trait;
use etcd_client::ConnectOptions;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

fn default_keep_alive_while_idle() -> bool {
    true
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EtcdConf {
    #[serde(default)]
    pub endpoints: Vec<String>,
    /// user is a pair values of name and password
    #[serde(default)]
    pub user: Option<(String, String)>,
    /// Whether send keep alive pings even there are no active streams.
    #[serde(default = "default_keep_alive_while_idle")]
    pub keep_alive_while_idle: bool,
}

impl Default for EtcdConf {
    fn default() -> Self {
        let endpoints: Vec<String> = optional("ETCD_ENDPOINTS", "127.0.0.1:2379")
            .split('|')
            .map(ToOwned::to_owned)
            .collect();

        Self {
            endpoints,
            user: None,
            keep_alive_while_idle: default_keep_alive_while_idle(),
        }
    }
}

pub struct Etcd(EtcdConf);

impl Etcd {
    pub fn new(conf: EtcdConf) -> Self {
        Self(conf)
    }
}

#[async_trait]
impl Middleware for Etcd {
    type Client = etcd_client::Client;
    type Error = etcd_client::Error;

    async fn make_client(&self) -> Result<Self::Client, Self::Error> {
        let options = match self.0.user.as_ref() {
            None => ConnectOptions::new().with_keep_alive_while_idle(self.0.keep_alive_while_idle),
            Some((name, password)) => ConnectOptions::new()
                .with_user(name, password)
                .with_keep_alive_while_idle(self.0.keep_alive_while_idle),
        };

        etcd_client::Client::connect(self.0.endpoints.deref(), Some(options)).await
    }
}

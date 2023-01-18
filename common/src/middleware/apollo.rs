use crate::config::env::{optional, optional_some, require};
use crate::middleware::Middleware;
use async_trait::async_trait;
use kosei::{ApolloClient, ConfigType};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct ApolloConf {
    pub addr: String,
    pub appid: String,
    pub namespace: String,
    pub config_type: String,
    pub cluster_name: String,
    pub secret: Option<String>,
}

impl Default for ApolloConf {
    fn default() -> Self {
        Self {
            addr: require("APOLLO_ADDR"),
            appid: require("APOLLO_APPID"),
            namespace: require("APOLLO_APPID"),
            config_type: optional("APOLLO_CONFIG_TYPE", "yaml"),
            cluster_name: optional("APOLLO_CLUSTER_NAME", "default"),
            secret: optional_some("APOLLO_SECRET"),
        }
    }
}

fn parse_config_type(typ: &str) -> ConfigType {
    match &*typ.to_lowercase() {
        "toml" => ConfigType::TOML,
        "json" => ConfigType::JSON,
        _ => ConfigType::YAML,
    }
}

pub struct Apollo(ApolloConf);

impl Apollo {
    pub fn new(conf: ApolloConf) -> Self {
        Self(conf)
    }
}

#[async_trait]
impl Middleware for Apollo {
    type Client = ApolloClient;
    type Error = Infallible;

    async fn make_client(&self) -> Result<Self::Client, Self::Error> {
        let conf = &self.0;
        Ok(ApolloClient::new(&conf.addr)
            .appid(&conf.appid)
            .cluster(&conf.cluster_name)
            .namespace(&conf.namespace, parse_config_type(&conf.config_type))
            .some_secret(conf.secret.as_deref()))
    }
}

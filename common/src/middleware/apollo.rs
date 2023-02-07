use crate::config::env::{optional, optional_some, require};
use crate::middleware::Middleware;
use async_trait::async_trait;
use kosei::{ApolloClient, ConfigType};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

fn default_addr() -> String {
    require("APOLLO_ADDR")
}

fn default_appid() -> String {
    require("APOLLO_APPID")
}

fn default_namespace() -> String {
    require("APOLLO_APPID")
}

fn default_config_type() -> String {
    optional("APOLLO_CONFIG_TYPE", "yaml")
}

fn default_cluster_name() -> String {
    optional("APOLLO_CLUSTER_NAME", "default")
}

fn default_secret() -> Option<String> {
    optional_some("APOLLO_SECRET")
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ApolloConf {
    #[serde(default = "default_addr")]
    pub addr: String,
    #[serde(default = "default_appid")]
    pub appid: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    #[serde(default = "default_config_type")]
    pub config_type: String,
    #[serde(default = "default_cluster_name")]
    pub cluster_name: String,
    #[serde(default = "default_secret")]
    pub secret: Option<String>,
}

impl Default for ApolloConf {
    fn default() -> Self {
        Self {
            addr: default_addr(),
            appid: default_appid(),
            namespace: default_namespace(),
            config_type: default_config_type(),
            cluster_name: default_cluster_name(),
            secret: default_secret(),
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

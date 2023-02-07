use super::env::*;
use super::*;
use names::Generator;
use serde::{Deserialize, Serialize};

pub trait ServiceConfig {
    type RestService: ConfigType;
    type GrpcService: ConfigType;
    type Service: ConfigType;
}

fn default_name() -> String {
    let mut generator = Generator::default();
    optional("SERVICE_NAME", generator.next().unwrap())
}

fn default_listen_addr() -> String {
    optional("LISTEN_ADDR", "0.0.0.0:3000")
}

fn default_discover_addr() -> String {
    optional("DISCOVER_ADDR", "http://127.0.0.1:3000")
}

fn default_timeout() -> u64 {
    30
}

fn default_concurrency_limit() -> usize {
    5120
}

fn default_load_shed() -> bool {
    false
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct ServiceConf {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default = "default_discover_addr")]
    pub discover_addr: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_concurrency_limit")]
    pub concurrency_limit: usize,
    #[serde(default = "default_load_shed")]
    pub load_shed: bool, // sheds load when the inner service isn't ready.
}

impl Default for ServiceConf {
    fn default() -> Self {
        Self {
            name: default_name(),
            listen_addr: default_listen_addr(),
            discover_addr: default_discover_addr(),
            timeout: default_timeout(),
            concurrency_limit: default_concurrency_limit(),
            load_shed: default_load_shed(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RestServiceConf {
    #[serde(default)]
    pub service: ServiceConf,
    #[serde(default)]
    pub cert_file: Option<String>,
    #[serde(default)]
    pub key_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GrpcServiceConf {
    #[serde(default)]
    pub service: ServiceConf,
    #[serde(default)]
    pub health_check: bool,
}

impl ServiceConfig for Config {
    type RestService = RestServiceConf;
    type GrpcService = GrpcServiceConf;
    type Service = ServiceConf;
}

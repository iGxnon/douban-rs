use super::env::*;
use super::*;
use names::Generator;
use serde::{Deserialize, Serialize};

pub trait ServiceConfig {
    type ApiService: ConfigType;
    type GrpcService: ConfigType;
}

fn random_name() -> String {
    let mut generator = Generator::default();
    generator.next().unwrap()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ServiceConf {
    pub domain: String,
    pub name: String,
    pub listen_addr: String,
    pub discover_addr: String,
    pub timeout: u64,
    pub concurrency_limit: usize,
    pub load_shed: bool, // sheds load when the inner service isn't ready.
}

impl Default for ServiceConf {
    fn default() -> Self {
        let name = optional("SERVICE_NAME", random_name());
        let domain = optional("SERVICE_DOMAIN", "sys"); // default domain is `sys`
        let discover_addr = optional(
            "DISCOVER_ADDR",
            format!("{}_{}:3000", domain.as_str(), name.as_str()),
        );
        Self {
            discover_addr,
            domain,
            name,
            listen_addr: optional("LISTEN_ADDR", "0.0.0.0:3000"),
            timeout: 30,
            concurrency_limit: 5120,
            load_shed: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ApiServiceConf {
    pub service: ServiceConf,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GrpcServiceConf {
    pub service: ServiceConf,
    pub health_check: bool,
}

impl ServiceConfig for Config {
    type ApiService = ApiServiceConf;
    type GrpcService = GrpcServiceConf;
}

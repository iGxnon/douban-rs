use super::env::*;
use super::*;
use crate::define_config;
use names::Generator;
use serde::Serialize;

pub trait ServiceConfig {
    type RestService: ConfigType;
    type GrpcService: ConfigType;
    type Service: ConfigType;
}

define_config! {
    #[derive(Serialize, Debug)]
    pub ServiceConf {
        #[default_name = "default_name"]
        pub name -> String {
            let mut generator = Generator::default();
            optional("SERVICE_NAME", generator.next().unwrap())
        },
        #[default_listen_addr = "default_listen_addr"]
        pub listen_addr -> String {
            optional("LISTEN_ADDR", "0.0.0.0:3000")
        },
        #[default_discover_addr = "default_discover_addr"]
        pub discover_addr -> String {
            optional("DISCOVER_ADDR", "http://127.0.0.1:3000")
        },
        #[default_timeout = "default_timeout"]
        pub timeout -> u64 {
            30
        },
        #[default_concurrency_limit = "default_concurrency_limit"]
        pub concurrency_limit -> usize {
            5120
        },
        #[default_load_shed = "default_load_shed"]
        pub load_shed -> bool {
            false
        }
    }
}

define_config! {
    #[derive(Serialize, Debug)]
    pub RestServiceConf (
        pub service: ServiceConf,
        pub cert_file: Option<String>,
        pub key_file: Option<String>,
    )
}

define_config! {
    #[derive(Serialize, Debug)]
    pub GrpcServiceConf (
        pub service: ServiceConf,
        pub health_check: bool,
    )
}

impl ServiceConfig for Config {
    type RestService = RestServiceConf;
    type GrpcService = GrpcServiceConf;
    type Service = ServiceConf;
}

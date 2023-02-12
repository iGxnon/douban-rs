use crate::config::env::optional;
use crate::define_config;
use crate::middleware::Middleware;
use async_trait::async_trait;
use serde::Serialize;

define_config! {
    #[derive(Serialize, Debug)]
    pub RedisConf {
        #[default_dsn = "default_dsn"]
        pub dsn -> String {
            optional("APP_REDIS", "redis://127.0.0.1/")
        }
    }
}

pub struct Redis(RedisConf);

impl Redis {
    pub fn new(conf: RedisConf) -> Self {
        Self(conf)
    }
}

#[async_trait]
impl Middleware for Redis {
    type Client = redis::Client;
    type Error = redis::RedisError;

    async fn make_client(&self) -> Result<Self::Client, Self::Error> {
        redis::Client::open(&*self.0.dsn)
    }
}

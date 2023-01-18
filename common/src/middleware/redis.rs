use crate::middleware::Middleware;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct RedisConf {
    pub dsn: String,
}

impl Default for RedisConf {
    fn default() -> Self {
        Self {
            dsn: "redis://127.0.0.1/".to_string(),
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

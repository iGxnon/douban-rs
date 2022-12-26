use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::OnceCell;
use redis::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

pub mod command;
pub mod error;
pub mod model;
pub mod query;

pub const SERVICE_NAME: &str = "auth/token";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub hmac_key: String,                 // jwt key (private HS256)
    pub clients: HashMap<String, String>, // sid -> s_secret(32bytes)
    pub exp_delta: HashMap<String, i64>,  // sid -> exp_delta
    pub refresh_delta_rate: i64,
    pub redis_dsn: String, // [redis|rediss(tls)]://username:password@redis_domain:redis_port/db_number
    pub pub_jwks: JwkSet,
    #[serde(default)]
    pub encode_pems: HashMap<String, String>, // jwt encode key (public RSA pem)
    #[serde(default)]
    pub health_check: bool,
    pub listen_addr: String,
    pub endpoint: String,
    pub etcd_clients: Vec<String>,
    #[serde(default = "default_grant_ttl")]
    pub etcd_grant_ttl: i64,
    #[serde(default = "default_keepalive_interval")]
    pub etcd_keepalive_interval: u64,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_concurrency_limit")]
    pub concurrency_limit: usize,
}

fn default_grant_ttl() -> i64 {
    60
}

fn default_keepalive_interval() -> u64 {
    20
}

fn default_timeout() -> u64 {
    30
}

fn default_concurrency_limit() -> usize {
    256
}

pub struct Resolver(Config);

static REDIS_STORE: OnceCell<RedisStore> = OnceCell::new();
static DECODE_KEY: OnceCell<DecodingKey> = OnceCell::new();
static ENCODE_KEY: OnceCell<EncodingKey> = OnceCell::new();
static ENCODE_PEM_KEY: OnceCell<HashMap<String, EncodingKey>> = OnceCell::new();

pub(in crate::domain) struct RedisStore(Client);

impl Deref for RedisStore {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Resolver {
    pub fn new(config: Config) -> Self {
        Resolver(config)
    }

    pub(in crate::domain) fn config(&self) -> &Config {
        &self.0
    }

    pub(in crate::domain) fn redis(&self) -> &RedisStore {
        REDIS_STORE.get_or_init(|| {
            RedisStore(Client::open(self.0.redis_dsn.to_owned()).expect("cannot parse redis dsn"))
        })
    }

    pub(in crate::domain) fn decode_key(&self) -> &DecodingKey {
        DECODE_KEY.get_or_init(|| DecodingKey::from_secret(self.0.hmac_key.as_bytes()))
    }

    pub(in crate::domain) fn encode_key(&self) -> &EncodingKey {
        ENCODE_KEY.get_or_init(|| EncodingKey::from_secret(self.0.hmac_key.as_bytes()))
    }

    pub(in crate::domain) fn encode_pem_key(&self, kid: &str) -> Option<&EncodingKey> {
        ENCODE_PEM_KEY
            .get_or_init(|| {
                let mut map = HashMap::new();
                for (kid, pem) in &self.0.encode_pems {
                    let key = EncodingKey::from_rsa_pem(pem.as_bytes())
                        .unwrap_or_else(|_| panic!("unexpect pem {}", kid));
                    map.insert(kid.clone(), key);
                }
                map
            })
            .get(kid)
    }
}

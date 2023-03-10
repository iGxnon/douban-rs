pub mod token;

use crate::auth::rpc::token::TokenService;
use base64::Engine;
use common::config::{middleware::MiddlewareConfig, register, service::ServiceConfig, Config};
use common::infra::{Resolver, Target};
use common::registry::{EtcdRegistry, ServiceRegister};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use proto::pb::auth::token::v1::token_service_server::TokenServiceServer;
use r2d2::PooledConnection;
use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tonic::transport::Server;
use tower::load_shed::LoadShedLayer;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

fn random_oct_key() -> String {
    let key: [u8; 32] = random();
    base64::prelude::BASE64_STANDARD.encode(key)
}

fn default_refresh_ratio() -> f32 {
    3.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenConfig {
    #[serde(default)]
    pub service_conf: <Config as ServiceConfig>::GrpcService,
    #[serde(default)]
    pub redis: <Config as MiddlewareConfig>::Redis,
    #[serde(default)]
    pub etcd: <Config as MiddlewareConfig>::Etcd,
    #[serde(default = "random_oct_key")]
    pub oct_key: String,
    #[serde(default = "default_refresh_ratio")]
    pub refresh_ratio: f32,
    #[serde(default)]
    pub expires: HashMap<String, u64>,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            service_conf: Default::default(),
            redis: Default::default(),
            etcd: Default::default(),
            oct_key: random_oct_key(),
            refresh_ratio: default_refresh_ratio(),
            expires: Default::default(),
        }
    }
}

type Register<T> = register::Register<TokenConfig, T>;
type RefRegister<T> = Register<&'static T>;

#[derive(Clone)]
pub struct TokenResolver {
    conf: TokenConfig,
    encode_key: RefRegister<EncodingKey>,
    decode_key: RefRegister<DecodingKey>,
    redis: RefRegister<r2d2::Pool<redis::Client>>,
}

impl Resolver for TokenResolver {
    const TARGET: Target = Target::GRPC;
    const DOMAIN: &'static str = "token";
    type Config = TokenConfig;

    fn conf(&self) -> &Self::Config {
        &self.conf
    }
}

impl TokenResolver {
    pub fn new(conf: TokenConfig) -> Self {
        Self {
            conf,
            encode_key: Register::once_ref(|conf| {
                EncodingKey::from_base64_secret(conf.oct_key.as_str()).unwrap()
            }),
            decode_key: Register::once_ref(|conf| {
                DecodingKey::from_base64_secret(conf.oct_key.as_str()).unwrap()
            }),
            redis: Register::once_ref(|conf| {
                r2d2::Pool::new(
                    redis::Client::open(conf.redis.dsn.as_str()).expect("unexpect redis dsn"),
                )
                .expect("cannot create r2d2 pool")
            }),
        }
    }

    pub fn add_expire(&mut self, audience: impl Into<String>, expire: u64) {
        self.conf.expires.insert(audience.into(), expire);
    }

    pub async fn register_service(&self) {
        let registry = EtcdRegistry::register(
            self.conf.etcd.clone(),
            self.conf.service_conf.service.clone(),
        );
        let service_key = Self::service_key();
        registry
            .register_service(&service_key)
            .await
            .expect("cannot register service into etcd");
    }

    pub async fn serve(&self) -> Result<(), tonic::transport::Error> {
        let token_srv = TokenService(self.clone());
        let addr = self
            .conf
            .service_conf
            .service
            .listen_addr
            .parse()
            .expect("cannot parse a valid listen_addr");

        let layer = ServiceBuilder::new()
            .catch_panic()
            .trace_for_grpc()
            .option_layer(if self.conf.service_conf.service.load_shed {
                Some(LoadShedLayer::new())
            } else {
                None
            })
            .concurrency_limit(self.conf.service_conf.service.concurrency_limit);

        let serve = Server::builder()
            .timeout(Duration::from_secs(self.conf.service_conf.service.timeout))
            .layer(layer)
            .add_optional_service(if self.conf.service_conf.health_check {
                let (mut reporter, svc) = tonic_health::server::health_reporter();
                reporter
                    .set_serving::<TokenServiceServer<TokenService>>()
                    .await;
                Some(svc)
            } else {
                None
            })
            .add_service(TokenServiceServer::new(token_srv));

        serve.serve(addr).await
    }

    pub(crate) fn decode_key(&self) -> &'static DecodingKey {
        self.resolve(&self.decode_key)
    }

    pub(crate) fn encode_key(&self) -> &'static EncodingKey {
        self.resolve(&self.encode_key)
    }

    pub(crate) fn algorithm(&self) -> Algorithm {
        Algorithm::HS256
    }

    pub(crate) fn redis_conn(&self) -> PooledConnection<redis::Client> {
        let pool = self.resolve(&self.redis);
        pool.get()
            .expect("Cannot get redis connection from r2d2 pool")
    }
}

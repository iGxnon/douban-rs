use crate::domain::token::pb::token_service_server::TokenServiceServer;
use crate::rpc::token::TokenService;
use base64::Engine;
use common::config::env::optional_sensitive;
use common::config::{middleware::MiddlewareConfig, register, service::ServiceConfig, Config};
use common::discover::{EtcdDiscover, EtcdDiscoverConf};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;
use tonic::transport::Server;
use tower::load_shed::LoadShedLayer;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

pub mod command;
pub mod model;
pub mod query;

pub mod pb {
    tonic::include_proto!("douban.auth.token");
}

#[derive(Clone)]
pub(in crate::domain) struct RedisStore(redis::Client);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TokenConfig {
    service_conf: <Config as ServiceConfig>::GrpcService,
    redis: <Config as MiddlewareConfig>::Redis,
    etcd: <Config as MiddlewareConfig>::Etcd,
    oct_key: String,
    refresh_ratio: f32,
    expires: HashMap<String, u64>,
}

impl Default for TokenConfig {
    fn default() -> Self {
        fn random_oct_key() -> String {
            let key: [u8; 32] = random();
            base64::prelude::BASE64_STANDARD.encode(key)
        }

        Self {
            service_conf: Default::default(),
            redis: Default::default(),
            etcd: Default::default(),
            oct_key: optional_sensitive("APP_ENCODE_KEY", random_oct_key()),
            refresh_ratio: 3.0,
            expires: Default::default(),
        }
    }
}

type Register<T> = register::Register<TokenConfig, T>;

#[derive(Clone)]
pub struct Resolver {
    conf: TokenConfig,
    encode_key: Register<EncodingKey>,
    decode_key: Register<DecodingKey>,
    redis: Register<RedisStore>,
}

impl Resolver {
    pub fn new(conf: TokenConfig) -> Self {
        Self {
            conf,
            encode_key: Register::once(|conf| {
                EncodingKey::from_base64_secret(conf.oct_key.as_str()).unwrap()
            }),
            decode_key: Register::once(|conf| {
                DecodingKey::from_base64_secret(conf.oct_key.as_str()).unwrap()
            }),
            redis: Register::once(|conf| {
                RedisStore(
                    redis::Client::open(conf.redis.dsn.as_str()).expect("unexpect redis dsn"),
                )
            }),
        }
    }

    pub fn make_discover(&self) -> EtcdDiscover {
        let conf = EtcdDiscoverConf::new(
            self.conf.etcd.clone(),
            self.conf.service_conf.service.clone(),
        );
        EtcdDiscover::new(conf)
    }

    pub async fn make_serve(&self) -> impl Future<Output = Result<(), tonic::transport::Error>> {
        let token_srv = TokenService(self.clone());
        let addr = self
            .conf
            .service_conf
            .service
            .listen_addr
            .parse()
            .expect("a valid listen_addr");

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

        serve.serve(addr)
    }

    fn resolve<T>(&self, register: &Register<T>) -> T {
        register.register(&self.conf)
    }

    pub(in crate::domain) fn decode_key(&self) -> DecodingKey {
        self.resolve(&self.decode_key)
    }

    pub(in crate::domain) fn encode_key(&self) -> EncodingKey {
        self.resolve(&self.encode_key)
    }

    pub(in crate::domain) fn algorithm(&self) -> Algorithm {
        Algorithm::HS256
    }

    pub(in crate::domain) fn redis_store(&self) -> RedisStore {
        self.resolve(&self.redis)
    }
}

pub mod command;
pub mod model;
pub mod query;

use crate::auth::domain::token::TokenResolver;
use crate::user::rpc::user::UserService;
use base64::Engine;
use common::config::env::optional;
use common::config::middleware::MiddlewareConfig;
use common::config::service::ServiceConfig;
use common::config::Config;
use common::infra::{Resolver, Target};
use common::registry::{EtcdRegistry, ServiceDiscover, ServiceRegister};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use proto::pb::auth::token::v1::token_service_client::TokenServiceClient;
use proto::pb::user::sys::v1::user_service_server::UserServiceServer;
use r2d2::PooledConnection;
use rand::random;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::time::Duration;
use tonic::transport::{Channel, Server};
use tower::load_shed::LoadShedLayer;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

fn random_hash_key() -> String {
    let key: [u8; 32] = random();
    base64::prelude::BASE64_STANDARD.encode(key)
}

fn pg_dsn() -> String {
    optional("PG_DB", "postgres://root:@localhost/s_douban_rs")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default)]
    service_conf: <Config as ServiceConfig>::GrpcService,
    #[serde(default)]
    redis: <Config as MiddlewareConfig>::Redis,
    #[serde(default)]
    etcd: <Config as MiddlewareConfig>::Etcd,
    #[serde(default = "random_hash_key")]
    hash_secret: String,
    #[serde(default = "pg_dsn")]
    pg_dsn: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            service_conf: Default::default(),
            redis: Default::default(),
            etcd: Default::default(),
            hash_secret: random_hash_key(),
            pg_dsn: pg_dsn(),
        }
    }
}

type Register<T> = common::config::register::Register<UserConfig, T>;

#[derive(Clone)]
pub struct UserResolver {
    conf: UserConfig,
    token_client: TokenServiceClient<Channel>,
    hash_secret: Register<&'static String>,
    pg_pool: Register<&'static Pool<ConnectionManager<PgConnection>>>,
}

impl Resolver for UserResolver {
    const TARGET: Target = Target::GRPC;

    const DOMAIN: &'static str = "user";

    type Config = UserConfig;

    fn conf(&self) -> &Self::Config {
        &self.conf
    }
}

// basic role group
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RoleGroup {
    USER,
    ADMIN,
}

impl RoleGroup {
    pub fn name(&self) -> &str {
        match self {
            RoleGroup::USER => "user",
            RoleGroup::ADMIN => "admin",
        }
    }
}

impl UserResolver {
    pub async fn new(conf: UserConfig) -> Self {
        let (channel, tx) = Channel::balance_channel(64);
        let discover = EtcdRegistry::discover(conf.etcd.clone());
        let service_key = TokenResolver::service_key();
        discover
            .discover_to_channel(&service_key, tx)
            .await
            .expect("Cannot connect to etcd service.");
        Self {
            conf,
            token_client: TokenServiceClient::new(channel),
            hash_secret: Register::once_ref(|conf| conf.hash_secret.clone()),
            pg_pool: Register::once_ref(|conf| {
                Pool::new(ConnectionManager::new(&conf.pg_dsn)).unwrap()
            }),
        }
    }

    pub fn hash_secret(&self) -> &str {
        self.resolve(&self.hash_secret).deref()
    }

    pub fn pg_conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.resolve(&self.pg_pool)
            .get()
            .expect("Cannot get pg connection")
    }

    pub fn token_client(&self) -> TokenServiceClient<Channel> {
        self.token_client.clone()
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
            .expect("Cannot register service into etcd");
    }

    pub async fn serve(&self) -> Result<(), tonic::transport::Error> {
        let token_srv = UserService(self.clone());
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
                    .set_serving::<UserServiceServer<UserService>>()
                    .await;
                Some(svc)
            } else {
                None
            })
            .add_service(UserServiceServer::new(token_srv));

        serve
            .serve_with_shutdown(addr, async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await
    }
}

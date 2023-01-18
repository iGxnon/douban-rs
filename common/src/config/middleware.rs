use super::*;

pub trait MiddlewareConfig {
    type Etcd: ConfigType;
    type Consul: ConfigType;
    type Apollo: ConfigType;
    type Redis: ConfigType;
}

impl MiddlewareConfig for Config {
    type Etcd = crate::middleware::etcd::EtcdConf;
    type Consul = crate::middleware::consul::ConsulConf;
    type Apollo = crate::middleware::apollo::ApolloConf;
    type Redis = crate::middleware::redis::RedisConf;
}

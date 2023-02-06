use super::*;

pub trait DiscoverConfig {
    type Etcd: ConfigType;
    type Consul: ConfigType;
}

impl DiscoverConfig for Config {
    type Etcd = crate::registry::EtcdRegistryConf;
    type Consul = crate::registry::ConsulRegistryConf;
}

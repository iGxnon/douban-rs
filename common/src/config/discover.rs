use super::*;

pub trait DiscoverConfig {
    type Etcd: ConfigType;
    type Consul: ConfigType;
}

impl DiscoverConfig for Config {
    type Etcd = crate::discover::EtcdDiscoverConf;
    type Consul = crate::discover::ConsulDiscoverConf;
}

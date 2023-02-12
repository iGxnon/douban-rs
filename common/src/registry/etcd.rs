use super::*;
use crate::middleware::etcd::Etcd;
use crate::middleware::Middleware;
use etcd_client::{EventType, GetOptions, PutOptions, WatchOptions};
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tonic::transport::Endpoint;
use tower::discover::Change;
use tracing::Instrument;
use tracing::{info, trace, warn};
use tracing_attributes::instrument;

#[derive(Debug, Default)]
pub struct EtcdRegistry(EtcdRegistryOption);

impl EtcdRegistry {
    pub fn new(conf: EtcdRegistryOption) -> Self {
        Self(conf)
    }

    pub fn discover(etcd: EtcdConf) -> Self {
        Self(EtcdRegistryOption::discover(etcd))
    }

    pub fn register(etcd: EtcdConf, service: ServiceConf) -> Self {
        Self(EtcdRegistryOption::register(etcd, service))
    }
}

#[async_trait]
impl ServiceRegister<String> for EtcdRegistry {
    type Error = etcd_client::Error;

    #[instrument(err, skip_all)]
    async fn register_service(&self, domain: &'static str) -> Result<(), Self::Error> {
        let (etcd, service, grant_ttl, keep_alive_interval) = match &self.0 {
            EtcdRegistryOption::Register {
                etcd,
                service,
                grant_ttl,
                keep_alive_interval,
            } => (etcd, service, *grant_ttl, *keep_alive_interval),
            EtcdRegistryOption::Discover { .. } => {
                panic!("Cannot register service_old with a discover config")
            }
        };

        debug_assert!(grant_ttl > keep_alive_interval as i64);

        let etcd = Etcd::new(etcd.clone());
        let mut client = etcd.make_client().await?;

        let lease_id = client.lease_grant(grant_ttl, None).await?.id();
        let (mut keeper, _) = client.lease_keep_alive(lease_id).await?;

        let task = async move {
            let mut tick = tokio::time::interval(Duration::from_secs(keep_alive_interval));
            loop {
                tick.tick().await;
                if let Err(err) = keeper.keep_alive().await {
                    warn!("keep lease alive failed cause err: {}", err);
                    break;
                }
                trace!("kept lease alive");
            }
        }
        .in_current_span();

        tokio::spawn(task);

        let name = service.name.as_str();
        let discover_addr = service.discover_addr.as_str();

        client
            .put(
                format!("{}:{}", domain, name),
                discover_addr,
                Some(PutOptions::new().with_lease(lease_id)),
            )
            .await?;

        Ok(())
    }
}

#[async_trait]
impl ServiceDiscover<String> for EtcdRegistry {
    type Error = etcd_client::Error;

    #[instrument(err, skip_all)]
    async fn discover_to_channel(
        &self,
        domain: &'static str,
        tx: Sender<Change<String, Endpoint>>,
    ) -> Result<(), Self::Error> {
        let etcd_conf = match &self.0 {
            EtcdRegistryOption::Register { etcd, .. } => etcd,
            EtcdRegistryOption::Discover { etcd } => etcd,
        };
        let etcd = Etcd::new(etcd_conf.clone());
        let mut client = etcd.make_client().await?;

        let (mut watcher, mut stream) = client
            .watch(domain, Some(WatchOptions::new().with_prefix()))
            .await?;
        watcher.request_progress().await.unwrap();

        let watch_id = watcher.watch_id();
        trace!("create a watch id {}", watch_id);

        let res = client
            .get(domain, Some(GetOptions::new().with_prefix()))
            .await?;

        info!(
            "initial discover {} services from domain '{}'",
            res.count(),
            domain
        );

        for kv in res.kvs() {
            let key = kv.key_str().unwrap();
            let value = kv.value_str().unwrap();

            if let Ok(endpoint) = Endpoint::from_str(value) {
                let _ = tx.send(Change::Insert(key.to_string(), endpoint)).await;
            } else {
                warn!(
                    "unexpected service_old endpoint {}, cannot parse it to an Endpoint",
                    value
                );
            }
        }

        let task = async move {
            while let Ok(Some(resp)) = stream.message().await {
                if resp.canceled() {
                    warn!(
                        "watcher has been canceled, reason: {}",
                        resp.cancel_reason()
                    );
                    break;
                }
                if resp.created() {
                    trace!("watcher create a new watch request");
                }

                for event in resp.events() {
                    match event.event_type() {
                        EventType::Put => {
                            if let Some(kv) = event.kv() {
                                let key = kv.key_str().unwrap();
                                let value = kv.value_str().unwrap();

                                if kv.version() == 1 {
                                    trace!("discover a new service_old {}: {}", key, value);
                                } else {
                                    trace!("service_old {} changed its endpoint to {}", key, value)
                                }

                                if let Ok(endpoint) = Endpoint::from_str(value) {
                                    let _ =
                                        tx.send(Change::Insert(key.to_string(), endpoint)).await;
                                } else {
                                    warn!("unexpected service_old endpoint {}, cannot parse it to an Endpoint", value);
                                }
                            }
                        }
                        EventType::Delete => {
                            if let Some(kv) = event.kv() {
                                let key = kv.key_str().unwrap();
                                trace!("service_old {} is going down", key);

                                let _ = tx.send(Change::Remove(key.to_string())).await;
                            }
                        }
                    }
                }
            }
        }.in_current_span();

        tokio::spawn(task);

        Ok(())
    }
}

// #[tokio::test]
// async fn test_discover() {
//     tracing_subscriber::fmt::init();
//     let etcd = EtcdConf::default();
//     let service_old = ServiceConf::default();
//     let conf = EtcdRegistryConf::register(etcd, service_old);
//     let registry = EtcdRegistry::new(conf);
//     let ok = registry.register_service("sys").await;
//     println!("{:?}", registry);
//     println!("{:?}", ok);
// }

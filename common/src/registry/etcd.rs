use super::*;
use crate::middleware::etcd::Etcd;
use crate::middleware::Middleware;
use etcd_client::{EventType, GetOptions, PutOptions, WatchOptions};
use log::warn;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tonic::transport::Endpoint;
use tower::discover::Change;
use tracing::Instrument;
use tracing::{debug, error, info};
use tracing_attributes::instrument;

#[derive(Debug, Default)]
pub struct EtcdDiscover(EtcdDiscoverConf);

impl EtcdDiscover {
    pub fn new(conf: EtcdDiscoverConf) -> Self {
        Self(conf)
    }
}

#[async_trait]
impl Discover<String> for EtcdDiscover {
    type Error = etcd_client::Error;

    #[instrument(err, skip_all)]
    async fn register_service<R: DomainProvider>(
        &self,
        domain_provider: R,
    ) -> Result<(), Self::Error> {
        let grant_ttl = self.0.grant_ttl;
        let keep_alive_interval = self.0.keep_alive_interval;

        debug_assert!(grant_ttl > keep_alive_interval as i64);

        let etcd = Etcd::new(self.0.etcd.clone());
        let mut client = etcd.make_client().await?;

        let lease_id = client.lease_grant(grant_ttl, None).await?.id();
        let (mut keeper, _) = client.lease_keep_alive(lease_id).await?;

        let task = async move {
            let mut tick = tokio::time::interval(Duration::from_secs(keep_alive_interval));
            loop {
                tick.tick().await;
                if let Err(err) = keeper.keep_alive().await {
                    error!("keep lease alive failed cause err: {}", err);
                    break;
                }
                debug!("kept lease alive");
            }
        }
        .in_current_span();

        tokio::spawn(task);

        let domain = domain_provider.domain();
        let name = self.0.service.name.as_str();
        let discover_addr = self.0.service.discover_addr.as_str();

        client
            .put(
                format!("{}:{}", domain, name),
                discover_addr,
                Some(PutOptions::new().with_lease(lease_id)),
            )
            .await?;

        Ok(())
    }

    #[instrument(err, skip_all)]
    async fn discover_to_channel<R: DomainProvider>(
        &self,
        domain_provider: R,
        tx: Sender<Change<String, Endpoint>>,
    ) -> Result<(), Self::Error> {
        let etcd = Etcd::new(self.0.etcd.clone());
        let mut client = etcd.make_client().await?;

        let domain = domain_provider.domain();

        let (mut watcher, mut stream) = client
            .watch(domain, Some(WatchOptions::new().with_prefix()))
            .await?;
        watcher.request_progress().await.unwrap();

        let watch_id = watcher.watch_id();
        info!("create a watch id {}", watch_id);

        let res = client
            .get(domain, Some(GetOptions::new().with_prefix()))
            .await?;

        info!("initial discover {} services", res.count());

        let task = async move {

            for kv in res.kvs() {
                let key = kv.key_str().unwrap();
                let value = kv.value_str().unwrap();

                if let Ok(endpoint) = Endpoint::from_str(value) {
                    let _ =
                        tx.send(Change::Insert(key.to_string(), endpoint)).await;
                } else {
                    error!("unexpected service endpoint {}, cannot parse it to an Endpoint", value);
                }
            }

            while let Ok(Some(resp)) = stream.message().await {
                if resp.canceled() {
                    warn!(
                        "watcher has been canceled, reason: {}",
                        resp.cancel_reason()
                    );
                    break;
                }
                if resp.created() {
                    info!("watcher create a new watch request");
                }

                for event in resp.events() {
                    match event.event_type() {
                        EventType::Put => {
                            if let Some(kv) = event.kv() {
                                let key = kv.key_str().unwrap();
                                let value = kv.value_str().unwrap();

                                if kv.version() == 1 {
                                    info!("discover a new service {}: {}", key, value);
                                } else {
                                    info!("service {} changed its endpoint to {}", key, value)
                                }

                                if let Ok(endpoint) = Endpoint::from_str(value) {
                                    let _ =
                                        tx.send(Change::Insert(key.to_string(), endpoint)).await;
                                } else {
                                    error!("unexpected service endpoint {}, cannot parse it to an Endpoint", value);
                                }
                            }
                        }
                        EventType::Delete => {
                            if let Some(kv) = event.kv() {
                                let key = kv.key_str().unwrap();
                                info!("service {} is going down", key);

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

#[tokio::test]
async fn test_discover() {
    tracing_subscriber::fmt::init();
    let etcd = EtcdConf::default();
    let service = ServiceConf::default();
    let conf = EtcdDiscoverConf::new(etcd, service);
    let discover = EtcdDiscover::new(conf);
    let ok = discover.register_service("sys").await;
    println!("{:?}", discover);
    println!("{:?}", ok);
}

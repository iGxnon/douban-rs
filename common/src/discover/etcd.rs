use etcd_client::{EventType, PutOptions, WatchOptions};
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tonic::transport::Endpoint;
use tower::discover::Change;
use tower::BoxError;
use tracing::{debug, error, info};

pub async fn register_service(
    client: &mut etcd_client::Client,
    service_name: &str,
    endpoint: &str,
    grant_ttl: i64,
    keep_alive_interval: u64,
) -> Result<(), BoxError> {
    debug_assert!(grant_ttl > keep_alive_interval as i64);

    let lease_id = client.lease_grant(grant_ttl, None).await?.id();
    let (mut keeper, _) = client.lease_keep_alive(lease_id).await?;

    tokio::spawn(async move {
        loop {
            if let Err(err) = keeper.keep_alive().await {
                error!("[discover] keep lease alive failed cause err: {}", err);
                break;
            }
            debug!("[discover] kept lease alive");
            tokio::time::sleep(Duration::from_secs(keep_alive_interval)).await;
        }
    });

    let id = rand::random::<u32>();
    debug!("[discover] generated service instance id: {}", id);

    client
        .put(
            format!("{}:{}", service_name, id),
            endpoint,
            Some(PutOptions::new().with_lease(lease_id)),
        )
        .await?;

    Ok(())
}

pub async fn channel_discover(
    client: &mut etcd_client::Client,
    service_name: &str,
    tx: Sender<Change<String, Endpoint>>,
) -> Result<(), BoxError> {
    let (mut watcher, mut stream) = client
        .watch(service_name, Some(WatchOptions::new().with_prefix()))
        .await?;
    watcher.request_progress().await.unwrap();

    let watch_id = watcher.watch_id();
    info!("[discover] create a watch id {}", watch_id);

    tokio::spawn(async move {
        while let Ok(Some(resp)) = stream.message().await {
            if resp.canceled() {
                info!(
                    "[discover] watcher has been canceled, reason: {}",
                    resp.cancel_reason()
                );
                break;
            }
            if resp.created() {
                info!("[discover] watcher create a new watch request");
            }

            for event in resp.events() {
                match event.event_type() {
                    EventType::Put => {
                        if let Some(kv) = event.kv() {
                            let key = kv.key_str().unwrap();
                            let value = kv.value_str().unwrap();

                            if kv.version() == 1 {
                                info!("[discover] discover a new service {}: {}", key, value);
                            } else {
                                info!(
                                    "[discover] service {} changed its endpoint to {}",
                                    key, value
                                )
                            }

                            if let Ok(endpoint) = Endpoint::from_str(value) {
                                let _ = tx.send(Change::Insert(key.to_string(), endpoint)).await;
                            } else {
                                error!("[discover] unexpected service endpoint {}, cannot parse it to an Endpoint", value);
                            }
                        }
                    }
                    EventType::Delete => {
                        if let Some(kv) = event.kv() {
                            let key = kv.key_str().unwrap();
                            info!("[discover] service {} is down", key);

                            let _ = tx.send(Change::Remove(key.to_string())).await;
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

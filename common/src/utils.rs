use crate::config::env::optional;
use crate::middleware::apollo::{Apollo, ApolloConf};
use crate::middleware::Middleware;
use kosei::Config;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn parse_config<E: serde::de::DeserializeOwned + Clone>(
    domain: &str,
) -> Result<E, Error> {
    let typ = optional("CONFIG_TYPE", "file");
    match typ.to_lowercase().as_str() {
        "file" => {
            let path = optional("CONFIG_PATH", "config");
            let path: &Path = path.as_ref();

            // parse config from directory with service_domain
            if path.is_dir() {
                let path = path.join(format!("{}.{}", domain, optional("CONFIG_FILETYPE", "yml")));
                return Ok(Config::<E>::from_file(path).into_inner());
            }

            Ok(Config::<E>::from_file(path).into_inner())
        }
        "apollo" => {
            let apollo = Apollo::new(ApolloConf::default());
            let client = apollo.make_client().await.unwrap();

            Ok(Config::<E>::from_apollo(&client).await?.into_inner())
        }
        _ => panic!("unsupported config type"),
    }
}

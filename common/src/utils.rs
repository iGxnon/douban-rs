use kosei::{ApolloClient, Config, ConfigType};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn parse_config<E: serde::de::DeserializeOwned + Clone>(
    service_name: &str,
) -> Result<E, Error> {
    let typ = std::env::var("CONFIG_TYPE")?;
    match typ.to_lowercase().as_str() {
        "file" => {
            let path = std::env::var("CONFIG_PATH")?;

            Ok(Config::<E>::from_file(path).into_inner())
        }
        "apollo" => {
            let appid = std::env::var("APOLLO_APP_ID")?;
            let cluster_name =
                std::env::var("APOLLO_CLUSTER_NAME").unwrap_or_else(|_| "default".into());
            let apollo_addr = std::env::var("APOLLO_ADDR")?;
            let apollo_secret = std::env::var("APOLLO_SECRET").ok();

            // read config
            let apollo_client = ApolloClient::new(&apollo_addr)
                .appid(&appid)
                .namespace(service_name, ConfigType::YAML)
                .cluster(&cluster_name)
                .some_secret(apollo_secret.as_deref());

            Ok(Config::<E>::from_apollo(&apollo_client).await?.into_inner())
        }
        _ => panic!("unsupported config type"),
    }
}

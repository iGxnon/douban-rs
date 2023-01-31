use user::rest::{RestConfig, RestResolver};

#[tokio::main]
async fn main() {
    let mut conf = RestConfig::default();
    conf.service_conf.service.listen_addr = "0.0.0.0:5001".to_string();
    println!("{}", serde_json::to_string_pretty(&conf).unwrap());
    let resolver = RestResolver::new(conf);
    resolver.serve().await;
}

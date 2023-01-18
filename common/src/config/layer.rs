use super::*;

// Detail config types trait
pub trait LayerConfig {
    type CookieAuth: ConfigType;
}

// Implement some type trait for root config
impl LayerConfig for Config {
    type CookieAuth = crate::layer::CookieAuthConf;
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    #[serde(default)]
    struct MyConfig {
        cookie_auth: <Config as LayerConfig>::CookieAuth,
    }

    #[test]
    fn test() {
        let config = MyConfig::default();
        println!("{:?}", config);
        let result = serde_yaml::from_str::<MyConfig>("");
        println!("{:?}", result);
    }
}

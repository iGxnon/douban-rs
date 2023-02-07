use once_cell::sync::OnceCell;
use std::sync::Arc;
use tracing::info;

pub mod layer;
pub mod middleware;
pub mod service;

// Root config type
pub struct Config;

// Prevent users from implementing the ConfigType trait.
mod private {
    pub trait Sealed {}
}

pub trait ConfigType: private::Sealed {}

impl<T> private::Sealed for T where T: Clone + for<'de> serde::de::Deserialize<'de> + Default {}

impl<T> ConfigType for T where T: private::Sealed {}

// Some useful functions for load string configuration from environment
pub mod env {

    use super::*;

    pub fn require(env_key: impl AsRef<str>) -> String {
        std::env::var(env_key.as_ref())
            .unwrap_or_else(|_| panic!("require an environment {}", env_key.as_ref()))
    }

    pub fn optional(env_key: impl AsRef<str>, default: impl ToString) -> String {
        std::env::var(env_key.as_ref()).unwrap_or_else(|_| {
            let ret = default.to_string();
            info!(
                "cannot found environment {}, use '{}' as default",
                env_key.as_ref(),
                ret
            );
            ret
        })
    }

    pub fn optional_some(env_key: impl AsRef<str>) -> Option<String> {
        std::env::var(env_key.as_ref()).ok().or({
            info!(
                "cannot found environment {}, use None as default",
                env_key.as_ref(),
            );
            None
        })
    }
}

pub mod register {
    use super::*;

    // Register grabbed a closure for generating values
    // specify the generic type C with your own config type
    #[derive(Clone)]
    pub struct Register<C: ConfigType, T>(Arc<dyn Fn(&C) -> T + Send + Sync>);

    impl<C: ConfigType, T> Register<C, T> {
        // Create a register that returns the same instance of a value.
        pub fn once(f: impl Fn(&C) -> T + Send + Sync + 'static) -> Self
        where
            T: Send + Sync + Clone + 'static,
        {
            let cell = OnceCell::new();
            Register(Arc::new(move |resolver| {
                cell.get_or_init(|| f(resolver)).clone()
            }))
        }

        // Use Box::leak to create a 'static lifetime register
        pub fn once_ref(f: impl Fn(&C) -> T + Send + Sync + 'static) -> Register<C, &'static T>
        where
            T: Sync + 'static,
        {
            let cell = OnceCell::new();
            Register(Arc::new(move |resolver| {
                cell.get_or_init(|| Box::leak(Box::new(f(resolver))) as &'static T)
            }))
        }

        // Create a register that returns a new instance of a value each time.
        pub fn factory(f: impl Fn(&C) -> T + Send + Sync + 'static) -> Self {
            Register(Arc::new(f))
        }

        pub fn register(&self, conf: &C) -> T {
            self.0(conf)
        }
    }
}

#[macro_export]
macro_rules! define_conf {
    (
        $(#[derive($($der:ident),+)])?
        $vis:vis struct $conf:ident {
            $(
                $(#($de:ident))?
                $(#[$dname:ident = $b:block, $iname:literal])?
                $fvis:vis $fname:ident: $typ:ty,
            )*
        }
    ) => {
        #[derive(Clone, serde::Deserialize, $($($der),+)?)]
        $vis struct $conf {
            $(
                $(#[serde(default = $iname)])?
                $(#[serde($de)])?
                $fvis $fname: $typ
            ),*
        }

        $(
            $(fn $dname() -> $typ $b)?
        )*

        impl Default for $conf {
            fn default() -> Self {
                Self {
                    $(
                        $($fname: Default::$de(),)?
                        $($fname: $dname(),)?
                    )*
                }
            }
        }
    };
    (
        $(#[derive($($der:ident),+)])?
        $vis:vis struct $conf:ident {
            $(
                $(#($de:ident))?
                $(#[$dname:ident = $e:expr, $iname:literal])?
                $fvis:vis $fname:ident: $typ:ty,
            )*
        }
    ) => {
        #[derive(Clone, serde::Deserialize, $($($der),+)?)]
        $vis struct $conf {
            $(
                $(#[serde(default = $iname)])?
                $(#[serde($de)])?
                $fvis $fname: $typ
            ),*
        }

        $(
            $(fn $dname() -> $typ { $e })?
        )*

        impl Default for $conf {
            fn default() -> Self {
                Self {
                    $(
                        $($fname: Default::$de(),)?
                        $($fname: $dname(),)?
                    )*
                }
            }
        }
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use super::middleware::MiddlewareConfig;
    use super::register::Register;
    use super::service::ServiceConfig;
    use super::Config;
    use base64::Engine;
    use rand::random;

    define_conf! {
        #[derive(Debug)]
        struct MyConfig {
            #(default)
            service_conf: <Config as ServiceConfig>::GrpcService,
            #(default)
            redis: <Config as MiddlewareConfig>::Redis,
            #[default_key = {
                let bytes: [u8; 32] = random();
                base64::prelude::BASE64_STANDARD.encode(bytes)
            }, "default_key"]
            encode_key: String,
        }
    }

    type MyRegister<T> = Register<MyConfig, T>;

    struct Resolver {
        conf: MyConfig,
        redis: MyRegister<redis::Client>,
        encode_key: MyRegister<Box<jsonwebtoken::EncodingKey>>,
    }

    impl Resolver {
        fn new(conf: MyConfig) -> Self {
            Self {
                conf,
                redis: MyRegister::once(|conf| {
                    redis::Client::open(conf.redis.dsn.as_str()).unwrap()
                }),
                encode_key: MyRegister::once(|conf| {
                    Box::new(
                        jsonwebtoken::EncodingKey::from_base64_secret(conf.encode_key.as_str())
                            .unwrap(),
                    )
                }),
            }
        }

        fn redis(&self) -> redis::Client {
            self.redis.register(&self.conf)
        }
    }

    #[test]
    fn test() {
        // Minimal configuration requires no configuration!
        let conf: MyConfig = serde_json::from_str("{}").unwrap();
        println!("{:?}", conf);
        let dsn_ref = MyRegister::once_ref(|conf| conf.redis.dsn.to_string());
        let dsn = dsn_ref.register(&conf) as &'static String;
        println!("{}", dsn);

        let resolver = Resolver::new(conf);

        let client = resolver.redis();
        println!("{:?}", client);
    }
}

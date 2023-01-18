use serde::Serialize;

pub mod error;
mod handler;
pub mod route;
mod types;

#[derive(Debug, Clone)]
pub struct Config {}

#[derive(Debug, Clone)]
pub struct Resolver(Config);

impl Resolver {
    pub fn new(config: Config) -> Self {
        Self(config)
    }
}

#[derive(Serialize, Debug)]
pub(in crate::api) struct Resp<T = ()> {
    msg: String,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl<T> Resp<T> {
    fn ok(data: T) -> Self {
        Self {
            msg: "success".to_string(),
            ok: true,
            data: Some(data),
        }
    }

    fn failed(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            ok: false,
            data: None,
        }
    }
}

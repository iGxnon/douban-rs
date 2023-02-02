pub mod command;
pub mod model;
pub mod query;

use common::infra::{Resolver, Target};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {}

type Register<T> = common::config::register::Register<UserConfig, T>;

pub struct UserResolver {
    conf: UserConfig,
}

impl Resolver for UserResolver {
    const TARGET: Target = Target::GRPC;

    const DOMAIN: &'static str = "user";

    type Config = UserConfig;

    fn conf(&self) -> &Self::Config {
        &self.conf
    }
}

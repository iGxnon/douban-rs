use crate::auth::layer::IdentityProvider;
use common::infra::Id;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct RegisterReq {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct LoginReq {
    pub(crate) identifier: String,
    pub(crate) password: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct BindReq {
    pub(crate) email: Option<String>,
    pub(crate) phone: Option<String>,
    pub(crate) github: Option<i64>,
}

#[derive(Clone)]
pub(crate) struct IdProvider;

pub(crate) struct User;
pub(crate) struct Group;
pub(crate) struct Extra;

pub(crate) type UserId = Id<User>;
pub(crate) type GroupId = Id<Group>;
pub(crate) type ExtraId = Id<Extra>;

impl IdentityProvider for IdProvider {
    type Id = UserId;
    type Group = GroupId;
    type Extra = ExtraId;
}

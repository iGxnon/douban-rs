use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(super) struct RegisterReq {
    pub(super) username: String,
    pub(super) password: String,
}

#[derive(Deserialize, Debug)]
pub(super) struct LoginReq {
    pub(super) identifier: String,
    pub(super) password: String,
}

#[derive(Deserialize, Debug)]
pub(super) struct BindReq {
    pub(super) email: String,
    pub(super) phone: String,
    pub(super) oauth_id: String,
}

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(in crate::api) struct SharedLoginReq {
    pub(in crate::api) username: String,
    pub(in crate::api) password: String,
}

#[derive(Deserialize, Debug)]
pub(in crate::api) struct BindReq {
    pub(in crate::api) email: String,
    pub(in crate::api) phone: String,
    pub(in crate::api) oauth_id: u64,
}

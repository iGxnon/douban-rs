use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(in crate::rest) struct RegisterReq {
    pub(in crate::rest) username: String,
    pub(in crate::rest) password: String,
}

#[derive(Deserialize, Debug)]
pub(in crate::rest) struct LoginReq {
    pub(in crate::rest) identifier: String,
    pub(in crate::rest) password: String,
}

#[derive(Deserialize, Debug)]
pub(in crate::rest) struct BindReq {
    pub(in crate::rest) email: String,
    pub(in crate::rest) phone: String,
    pub(in crate::rest) oauth_id: String,
}

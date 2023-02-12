use common::{infra::Command, status::prelude::*};
use proto::pb::user::sys::v1 as pb;

use crate::user::domain::user::UserResolver;

#[tracing::instrument(skip_all, err)]
async fn execute(req: pb::LoginReq) -> GrpcResult<pb::LoginRes> {
    Ok(pb::LoginRes {
        access: None,
        refresh: None,
    })
}

impl UserResolver {
    pub fn create_login(&self) -> impl Command<pb::LoginReq> + '_ {
        move |req: pb::LoginReq| async move { execute(req).await }
    }
}

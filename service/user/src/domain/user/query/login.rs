use common::{infra::Command, status::prelude::*};
use proto::pb::user::sys::v1 as pb;

use crate::domain::user::UserResolver;

#[tracing::instrument(skip_all, err)]
async fn execute(req: pb::LoginReq) -> GrpcResult<pb::LoginRes> {
    todo!()
}

impl UserResolver {
    pub fn create_login(&self) -> impl Command<pb::LoginReq> + '_ {
        move |req: pb::LoginReq| async move { todo!() }
    }
}

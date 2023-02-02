use common::{infra::Command, status::prelude::*};
use proto::pb::user::sys::v1 as pb;

use crate::domain::user::UserResolver;

#[tracing::instrument(skip_all, err)]
async fn execute(req: pb::BindReq) -> GrpcResult<pb::BindRes> {
    todo!()
}

impl UserResolver {
    pub fn create_bind(&self) -> impl Command<pb::BindReq> + '_ {
        move |req: pb::BindReq| async move { todo!() }
    }
}

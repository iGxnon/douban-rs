use common::{infra::Command, status::prelude::*};
use proto::pb::user::sys::v1 as pb;

use crate::domain::user::UserResolver;

#[tracing::instrument(skip_all, err)]
async fn execute(req: pb::RegisterReq) -> GrpcResult<pb::RegisterRes> {
    todo!()
}

impl UserResolver {
    pub fn create_register(&self) -> impl Command<pb::RegisterReq> + '_ {
        move |req: pb::RegisterReq| async move { todo!() }
    }
}

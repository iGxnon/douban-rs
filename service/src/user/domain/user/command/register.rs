use crate::user::domain::user::model::user::User;
use crate::user::rpc::{RoleGroup, UserResolver};
use common::{infra::Command, status::prelude::*};
use diesel::PgConnection;
use proto::pb::common::v1::EmptyRes;
use proto::pb::user::sys::v1 as pb;
use std::ops::DerefMut;

#[tracing::instrument(skip_all, err)]
async fn execute(
    req: pb::RegisterReq,
    secret: &str,
    conn: &mut PgConnection,
) -> GrpcResult<EmptyRes> {
    // TODO check req parameters.

    User::register(&req.username, &req.password, secret, RoleGroup::USER, conn)?;
    Ok(EmptyRes {})
}

impl UserResolver {
    pub(in crate::user) fn create_register(&self) -> impl Command<pb::RegisterReq> + '_ {
        move |req: pb::RegisterReq| async move {
            execute(req, self.hash_secret(), self.pg_conn().deref_mut()).await
        }
    }
}

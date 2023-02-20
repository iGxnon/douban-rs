use crate::user::domain::user::model::user::User;
use common::{infra::Command, status::prelude::*};
use diesel::PgConnection;
use proto::pb::common::v1::EmptyRes;
use proto::pb::user::sys::v1 as pb;
use std::ops::DerefMut;

use crate::user::domain::user::UserResolver;

#[tracing::instrument(skip_all, err)]
async fn execute(req: pb::BindReq, conn: &mut PgConnection) -> GrpcResult<EmptyRes> {
    // todo check req
    let mut user = User::query_identifier(&req.identifier, conn)?;
    if req.email.is_some() || req.phone.is_some() {
        user.bind(req.email, req.phone, conn)?;
    }
    if let Some(github) = req.github {
        user.bind_github(github.into(), conn)?;
    }
    Ok(EmptyRes {})
}

impl UserResolver {
    pub fn create_bind(&self) -> impl Command<pb::BindReq> + '_ {
        move |req: pb::BindReq| async move { execute(req, self.pg_conn().deref_mut()).await }
    }
}

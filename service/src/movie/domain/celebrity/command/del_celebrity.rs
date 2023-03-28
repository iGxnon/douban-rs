use diesel::PgConnection;
use common::infra::Command;
use common::status::prelude::*;
use proto::pb::common::v1::EmptyRes;
use proto::pb::movie::celebrity::v1 as pb;
use crate::movie::rpc::MovieResolver;

#[tracing::instrument(skip_all, err)]
async fn execute(req: pb::DelReq, conn: &mut PgConnection) -> GrpcResult<EmptyRes> {
    todo!()
}

impl MovieResolver {
    pub(in crate::movie) fn create_del_celebrity(&self) -> impl Command<pb::DelReq> + '_ {
        move |req: pb::DelReq| async move {
            execute(req, todo!())
        }
    }
}
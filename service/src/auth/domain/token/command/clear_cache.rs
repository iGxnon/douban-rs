use crate::auth::domain::token::model::token::{Token, TokenKind};
use crate::auth::domain::token::TokenResolver;
use common::infra::*;
use common::status::ext::GrpcResult;
use proto::pb::auth::token::v1 as pb;
use proto::pb::common::v1::EmptyRes;
use std::ops::DerefMut;
use tracing::instrument;

#[instrument(skip_all, err)]
async fn execute(req: pb::ClearCacheReq, conn: &mut redis::Connection) -> GrpcResult<EmptyRes> {
    Token::clear_cache(&req.sub, TokenKind::Access, conn)?;
    Token::clear_cache(&req.sub, TokenKind::Refresh, conn)?;
    Ok(EmptyRes {})
}

impl TokenResolver {
    pub fn create_clear_cache(&self) -> impl Command<pb::ClearCacheReq> + '_ {
        move |req: pb::ClearCacheReq| async move { execute(req, self.redis_conn().deref_mut()).await }
    }
}

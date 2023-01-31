use crate::domain::token::model::{Token, TokenKind};
use crate::domain::token::{RedisStore, TokenResolver};
use common::infra::*;
use common::status::ext::GrpcResult;
use proto::pb::auth::token::v1 as pb;
use tracing::{instrument, trace};

#[instrument(skip_all, err)]
async fn execute(req: pb::ClearCacheReq, store: RedisStore) -> GrpcResult<pb::ClearCacheRes> {
    trace!("Clear access token...");
    Command::execute(
        Token::with(req.sub.clone().into(), TokenKind::Access),
        Del::new(store.clone()),
    )
    .await?;
    trace!("Clear refresh token...");
    Command::execute(
        Token::with(req.sub.into(), TokenKind::Refresh),
        Del::new(store.clone()),
    )
    .await?;
    Ok(pb::ClearCacheRes {})
}

impl TokenResolver {
    pub fn create_clear_cache(&self) -> impl Command<pb::ClearCacheReq> + '_ {
        move |req: pb::ClearCacheReq| async move { execute(req, self.redis_store()).await }
    }
}

use crate::domain::token::model::{Token, TokenKind};
use crate::domain::token::TokenResolver;
use common::infra::*;
use common::status::ext::GrpcResult;
use proto::pb::auth::token::v1 as pb;
use tracing::{instrument, trace};

#[instrument(skip_all, err)]
async fn execute(
    req: pb::ClearCacheReq,
    store: &'static redis::Client,
) -> GrpcResult<pb::ClearCacheRes> {
    trace!("Clear access token...");
    Command::execute(
        Token::with(req.sub.clone().into(), TokenKind::Access),
        RedisDel::new(store),
    )
    .await?;
    trace!("Clear refresh token...");
    Command::execute(
        Token::with(req.sub.into(), TokenKind::Refresh),
        RedisDel::new(store),
    )
    .await?;
    Ok(pb::ClearCacheRes {})
}

impl TokenResolver {
    pub fn create_clear_cache(&self) -> impl Command<pb::ClearCacheReq> + '_ {
        move |req: pb::ClearCacheReq| async move { execute(req, self.redis_store()).await }
    }
}

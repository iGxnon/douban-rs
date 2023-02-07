use crate::domain::token::model::{Claim, Payload, Token, TokenId, TokenKind};
use crate::domain::token::TokenResolver;
use common::infra::*;
use common::invalid_argument;
use common::status::ext::GrpcResult;
use jsonwebtoken::{Algorithm, EncodingKey};
use proto::pb::auth::token::v1 as pb;
use std::collections::HashMap;
use tracing::{instrument, trace};

#[instrument(skip_all, err)]
async fn execute(
    req: pb::GenerateTokenReq,
    service_domain: &str,
    encode_key: &EncodingKey,
    algorithm: Algorithm,
    store: &'static redis::Client,
    refresh_ratio: f32,
    expires: &HashMap<String, u64>,
) -> GrpcResult<pb::GenerateTokenRes> {
    let token_id: TokenId = req.sub.as_str().into();
    trace!("Checking tokens if they are cached...");
    let cached_access = Query::execute(
        &Token::with(token_id.clone(), TokenKind::Access),
        RedisGet::new(store),
    )
    .await?;
    let cached_refresh = Query::execute(
        &Token::with(token_id.clone(), TokenKind::Refresh),
        RedisGet::new(store),
    )
    .await?;

    if cached_access.is_some() && cached_refresh.is_some() {
        trace!("Found cached tokens is redis, return them.");
        return Ok(pb::GenerateTokenRes {
            access: cached_access.map(Token::to_pb_exact),
            refresh: cached_refresh.map(Token::to_pb_exact),
        });
    }

    trace!("Generating new token pair...");
    let exp = *expires
        .get(req.aud.as_str())
        .ok_or_else(|| invalid_argument!("aud", "existed audience"))?;

    let now = jsonwebtoken::get_current_timestamp();
    let mut access_claim = Claim::builder(exp + now)
        .issue_at(now)
        .subject(req.sub.as_str())
        .audience(req.aud.as_str())
        .issuer(service_domain)
        .payload(Payload {
            id: token_id.clone(),
            kind: TokenKind::Access,
            detail: req.payload.clone(),
        });
    let exp = (exp as f32 * refresh_ratio) as u64;
    let mut refresh_claim = Claim::builder(exp + now)
        .issue_at(now)
        .subject(req.sub.as_str())
        .audience(req.aud.as_str())
        .issuer(service_domain)
        .payload(Payload {
            id: token_id.clone(),
            kind: TokenKind::Refresh,
            detail: req.payload.clone(),
        });

    if req.jti() {
        trace!("Assign uuid jti to tokens.");
        access_claim.uuid_jti();
        refresh_claim.uuid_jti();
    }

    let mut access_token = Token::new(token_id.clone(), TokenKind::Access, access_claim);
    let mut refresh_token = Token::new(token_id.clone(), TokenKind::Refresh, refresh_claim);

    trace!("Signing token...");
    let _ = access_token.sign(encode_key, algorithm)?;
    let _ = refresh_token.sign(encode_key, algorithm)?;

    trace!("Save tokens into cache");
    Command::execute(access_token.clone(), RedisSet::new(store)).await?;
    Command::execute(refresh_token.clone(), RedisSet::new(store)).await?;

    Ok(pb::GenerateTokenRes {
        access: Some(access_token.to_pb_exact()),
        refresh: Some(refresh_token.to_pb_exact()),
    })
}

impl TokenResolver {
    pub fn create_generate_token(&self) -> impl Command<pb::GenerateTokenReq> + '_ {
        move |req: pb::GenerateTokenReq| async move {
            execute(
                req,
                Self::DOMAIN,
                self.encode_key(),
                self.algorithm(),
                self.redis_store(),
                self.conf.refresh_ratio,
                &self.conf.expires,
            )
            .await
        }
    }
}

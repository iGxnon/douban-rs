use super::super::pb;
use crate::domain::token::model::{Claim, Payload, Token, TokenId, TokenKind};
use crate::domain::token::{RedisStore, Resolver};
use common::infra::*;
use common::invalid_argument;
use jsonwebtoken::{Algorithm, EncodingKey};
use std::collections::HashMap;
use tonic::Status;

impl Args for pb::GenerateTokenReq {
    type Output = Result<pb::GenerateTokenRes, Status>;
}

async fn execute(
    req: pb::GenerateTokenReq,
    service_domain: &str,
    encode_key: &EncodingKey,
    algorithm: Algorithm,
    store: RedisStore,
    refresh_ratio: f32,
    expires: &HashMap<String, u64>,
) -> Result<pb::GenerateTokenRes, Status> {
    let token_id: TokenId = req.sub.as_str().into();
    // check cache
    let cached_access = Query::execute(
        &Token::with(token_id.clone(), TokenKind::Access),
        GetFrom::new(store.clone()),
    )
    .await?;
    let cached_refresh = Query::execute(
        &Token::with(token_id.clone(), TokenKind::Refresh),
        GetFrom::new(store.clone()),
    )
    .await?;

    if cached_access.is_some() && cached_refresh.is_some() {
        // return if cached
        return Ok(pb::GenerateTokenRes {
            access: cached_access.map(Token::to_pb_exact),
            refresh: cached_refresh.map(Token::to_pb_exact),
        });
    }

    // generate new token
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
        access_claim.uuid_jti();
        refresh_claim.uuid_jti();
    }

    let mut access_token = Token::new(token_id.clone(), TokenKind::Access, access_claim);
    let mut refresh_token = Token::new(token_id.clone(), TokenKind::Refresh, refresh_claim);

    // sign token
    let _ = access_token.sign(encode_key, algorithm)?;
    let _ = refresh_token.sign(encode_key, algorithm)?;

    // save to cache
    Command::execute(access_token.clone(), SetInto::new(store.clone())).await?;
    Command::execute(refresh_token.clone(), SetInto::new(store.clone())).await?;

    Ok(pb::GenerateTokenRes {
        access: Some(access_token.to_pb_exact()),
        refresh: Some(refresh_token.to_pb_exact()),
    })
}

impl Resolver {
    pub fn create_generate_token(&self) -> impl Command<pb::GenerateTokenReq> + '_ {
        move |req: pb::GenerateTokenReq| async move {
            let service_domain = self.conf.service_conf.service.domain.as_str();
            execute(
                req,
                service_domain,
                &self.encode_key(),
                self.algorithm(),
                self.redis_store(),
                self.conf.refresh_ratio,
                &self.conf.expires,
            )
            .await
        }
    }
}

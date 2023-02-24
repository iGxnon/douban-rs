use crate::auth::domain::token::model::token::{Token, TokenKind};
use crate::auth::rpc::TokenResolver;
use common::infra::*;
use common::status::ext::GrpcResult;
use jsonwebtoken::{Algorithm, EncodingKey};
use proto::pb::auth::token::v1 as pb;
use std::collections::HashMap;
use std::ops::DerefMut;
use tracing::instrument;

#[instrument(skip_all, err)]
async fn execute(
    req: pb::GenerateTokenReq,
    domain: &str,
    encode_key: &EncodingKey,
    algorithm: Algorithm,
    refresh_ratio: f32,
    expires: &HashMap<String, u64>,
    conn: &mut redis::Connection,
) -> GrpcResult<pb::GenerateTokenRes> {
    let access_token = Token::get_or_generate(
        &req.sub,
        &req.aud,
        TokenKind::Access,
        req.jti(),
        req.payload.clone(),
        domain,
        encode_key,
        algorithm,
        refresh_ratio,
        expires,
        conn,
    )?;

    let refresh_token = Token::get_or_generate(
        &req.sub,
        &req.aud,
        TokenKind::Refresh,
        req.jti(),
        req.payload,
        domain,
        encode_key,
        algorithm,
        refresh_ratio,
        expires,
        conn,
    )?;

    Ok(pb::GenerateTokenRes {
        access: Some(access_token.into_pb()),
        refresh: Some(refresh_token.into_pb()),
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
                self.conf().refresh_ratio,
                &self.conf().expires,
                self.redis_conn().deref_mut(),
            )
            .await
        }
    }
}

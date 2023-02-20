use crate::auth::domain::token::model::claim;
use crate::auth::domain::token::model::claim::LEE_WAY;
use base64::Engine;
use common::status::prelude::*;
use common::{internal, invalid_argument};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use proto::pb::auth::token::v1 as pb;
use redis::Commands;
use serde::*;
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub type Claim = claim::Claim<Payload>;

pub struct Token {
    kind: TokenKind,
    raw_parts: (String, String), // (signature, message)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub kind: TokenKind,
    #[serde(flatten)]
    pub detail: Option<pb::Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Copy)]
pub enum TokenKind {
    #[serde(rename = "access")]
    Access,
    #[serde(rename = "refresh")]
    Refresh,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Access => write!(f, "access"),
            TokenKind::Refresh => write!(f, "refresh"),
        }
    }
}

macro_rules! expect_two {
    ($iter:expr) => {{
        let mut i = $iter;
        match (i.next(), i.next(), i.next()) {
            (Some(first), Some(second), None) => (first, second),
            _ => return Err(invalid_argument!("token", "valid jwt format with 3 dot").into()),
        }
    }};
}

impl Token {
    pub(in super::super) fn clear_cache(
        sub: &str,
        kind: TokenKind,
        conn: &mut redis::Connection,
    ) -> GrpcResult<()> {
        let key = format!("auth:token:{}:{}", sub, kind);
        conn.del(&key)
            .map_err(|_| internal!(format!("Failed to del key {}", key)))?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(in super::super) fn get_or_generate(
        sub: &str,
        aud: &str,
        kind: TokenKind,
        jti: bool,
        payload: Option<pb::Payload>,
        domain: &str,
        encode_key: &EncodingKey,
        algorithm: Algorithm,
        refresh_ratio: f32,
        expires: &HashMap<String, u64>,
        conn: &mut redis::Connection,
    ) -> GrpcResult<Self> {
        let key = format!("auth:token:{}:{}", sub, kind);
        let token_str: Option<String> = conn
            .get(&key)
            .map_err(|e| internal!(format!("Redis error, cannot get key {}, err: {}", key, e)))?;
        if let Some(value) = token_str {
            return Ok(Self {
                kind,
                raw_parts: expect_two!(value.rsplitn(2, '.').map(ToOwned::to_owned)),
            });
        }
        Self::generate(
            &key,
            sub,
            aud,
            kind,
            jti,
            domain,
            encode_key,
            algorithm,
            payload,
            refresh_ratio,
            expires,
            conn,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn generate(
        key: &str,
        sub: &str,
        aud: &str,
        kind: TokenKind,
        jti: bool,
        domain: &str,
        encode_key: &EncodingKey,
        algorithm: Algorithm,
        payload: Option<pb::Payload>,
        refresh_ratio: f32,
        expires: &HashMap<String, u64>,
        conn: &mut redis::Connection,
    ) -> GrpcResult<Self> {
        let exp = *expires
            .get(aud)
            .ok_or_else(|| invalid_argument!("aud", "an existed audience"))?;
        let exp = if kind == TokenKind::Refresh {
            (exp as f32 * refresh_ratio) as u64
        } else {
            exp
        };
        let now = jsonwebtoken::get_current_timestamp();
        let mut claim = Claim::builder(exp + now)
            .issue_at(now)
            .subject(sub)
            .audience(aud)
            .issuer(domain)
            .payload(Payload {
                kind,
                detail: payload,
            });
        if jti {
            claim.uuid_jti();
        }
        let header = jsonwebtoken::Header::new(algorithm);
        let signed = Self::sign(encode_key, &header, &claim)?;
        let (signature, message) = expect_two!(signed.rsplitn(2, '.').map(ToOwned::to_owned));
        let ex = max(exp, LEE_WAY);
        conn.set_ex(key, &signed, ex as usize)
            .map_err(|e| internal!(format!("Redis failed to set key {}, err: {}", key, e)))?;
        Ok(Self {
            kind,
            raw_parts: (signature, message),
        })
    }

    fn sign(
        encode_key: &EncodingKey,
        header: &jsonwebtoken::Header,
        claim: &Claim,
    ) -> GrpcResult<String> {
        let signed = jsonwebtoken::encode(header, claim, encode_key)
            .map_err(|_| internal!("cannot encode jwt with argument"))?;
        Ok(signed)
    }

    pub(in super::super) fn into_pb(self) -> pb::Token {
        let kind = match self.kind {
            TokenKind::Access => pb::TokenKind::Access,
            TokenKind::Refresh => pb::TokenKind::Refresh,
        };
        pb::Token {
            value: format!("{}.{}", self.raw_parts.1, self.raw_parts.0),
            kind: kind as i32,
        }
    }

    pub(in super::super) fn validate(
        &self,
        key: &DecodingKey,
        algorithm: Algorithm,
    ) -> GrpcResult<bool> {
        let signature = self.raw_parts.0.as_str();
        let message = self.raw_parts.1.as_str();
        let check = jsonwebtoken::crypto::verify(signature, message.as_ref(), key, algorithm)
            .map_err(|e| match e.into_kind() {
                ErrorKind::Base64(_) => invalid_argument!("token", "base64 encode signature"),
                _ => internal!(capture), // unreachable code reached, capture stacks trace and return
            })?;
        Ok(check)
    }

    pub(in super::super) fn kind(&self) -> TokenKind {
        self.kind
    }

    pub(in super::super) fn claim(&self) -> GrpcResult<Claim> {
        let (claim, _) = parse_message(&self.raw_parts.1)?;
        Ok(claim)
    }
}

#[inline]
fn parse_message(mess: &str) -> GrpcResult<(Claim, &str)> {
    let (claim, header) = expect_two!(mess.rsplitn(2, '.'));
    let decode = base64::prelude::BASE64_URL_SAFE_NO_PAD
        .decode(claim)
        .map_err(|_| invalid_argument!("token", "base64 encode message"))?;
    let claim = serde_json::from_slice::<Claim>(decode.as_slice())
        .map_err(|_| invalid_argument!("token", "valid json encode claim"))?;
    Ok((claim, header))
}

#[inline]
fn parse_raw(value: &str) -> GrpcResult<(Claim, &str, &str)> {
    let (signature, message) = expect_two!(value.rsplitn(2, '.'));
    let (claim, _) = parse_message(message)?;
    Ok((claim, signature, message))
}

impl FromStr for Token {
    type Err = GrpcStatus;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (claim, signature, message) = parse_raw(value)?;
        let payload = claim
            .as_payload()
            .ok_or_else(|| invalid_argument!("token", "with payload"))?;
        Ok(Self {
            kind: payload.kind,
            raw_parts: (signature.to_string(), message.to_string()),
        })
    }
}

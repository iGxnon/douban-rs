pub mod store;

use super::pb;
use base64::Engine;
use common::*;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::jwk::Jwk;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use tonic::Status;

const LEE_WAY: u64 = 60;

pub type TokenId = infra::Id<Token>;
pub type Claim = claim::Claim<Payload>;

type Result<T> = std::result::Result<T, Status>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    id: TokenId,
    claim: TokenClaim,
    raw_parts: (String, String),
}

#[derive(Debug, Clone, Default)]
struct Signature(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub id: TokenId,
    pub kind: TokenKind,
    #[serde(flatten)]
    pub detail: Option<pb::Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenKind {
    #[serde(rename = "access")]
    Access,
    #[serde(rename = "refresh")]
    Refresh,
}

impl From<TokenKind> for pb::TokenKind {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Access => pb::TokenKind::Access,
            TokenKind::Refresh => pb::TokenKind::Refresh,
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Access => write!(f, "access"),
            TokenKind::Refresh => write!(f, "refresh"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TokenClaim {
    Access(Claim),
    Refresh(Claim),
}

impl TokenClaim {
    fn inner(&self) -> &Claim {
        match self {
            TokenClaim::Access(token) => token,
            TokenClaim::Refresh(token) => token,
        }
    }

    fn new(claim: Claim, kind: TokenKind) -> Self {
        match kind {
            TokenKind::Access => Self::Access(claim),
            TokenKind::Refresh => Self::Refresh(claim),
        }
    }

    fn kind(&self) -> TokenKind {
        match self {
            TokenClaim::Access(_) => TokenKind::Access,
            TokenClaim::Refresh(_) => TokenKind::Refresh,
        }
    }
}

impl TryFrom<pb::Token> for Token {
    type Error = Status;

    fn try_from(value: pb::Token) -> Result<Self> {
        Token::from_pb(value)
    }
}

fn expect_two<I: Iterator>(mut split: I) -> Result<(I::Item, I::Item)> {
    match (split.next(), split.next(), split.next()) {
        (Some(one), Some(two), None) => Ok((one, two)),
        _ => Err(invalid_argument!("token", "valid jwt format with 3 dot")),
    }
}

impl FromStr for Token {
    type Err = Status;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let (claim, signature, message) = Token::parse_raw(value)?;
        let payload = claim
            .as_payload()
            .ok_or_else(|| invalid_argument!("token", "with payload"))?;
        let id = payload.id.clone();
        match payload.kind {
            TokenKind::Access => Ok(Self {
                id,
                claim: TokenClaim::Access(claim),
                raw_parts: (signature.to_string(), message.to_string()),
            }),
            TokenKind::Refresh => Ok(Self {
                id,
                claim: TokenClaim::Refresh(claim),
                raw_parts: (signature.to_string(), message.to_string()),
            }),
        }
    }
}

impl Token {
    pub fn new(id: TokenId, kind: TokenKind, claim: Claim) -> Self {
        Self {
            id,
            claim: TokenClaim::new(claim, kind),
            raw_parts: ("".to_string(), "".to_string()),
        }
    }

    pub fn kind(&self) -> TokenKind {
        self.claim.kind()
    }

    pub fn payload(&self) -> Option<&Payload> {
        self.claim.inner().as_payload()
    }

    pub fn claim(&self) -> &Claim {
        self.claim.inner()
    }

    pub fn with(id: TokenId, kind: TokenKind) -> Self {
        Self {
            id,
            claim: TokenClaim::new(Claim::new(0), kind),
            raw_parts: ("".to_string(), "".to_string()),
        }
    }

    pub fn from_pb(value: pb::Token) -> Result<Self> {
        match value.kind() {
            pb::TokenKind::Access => {
                let (claim, signature, message) = Token::parse_raw(value.value.as_str())?;
                let id = claim
                    .as_payload()
                    .ok_or_else(|| invalid_argument!("token", "with payload"))?
                    .id
                    .clone();
                Ok(Self {
                    id,
                    claim: TokenClaim::Access(claim),
                    raw_parts: (signature.to_string(), message.to_string()),
                })
            }
            pb::TokenKind::Refresh => {
                let (claim, signature, message) = Token::parse_raw(value.value.as_str())?;
                let id = claim
                    .as_payload()
                    .ok_or_else(|| invalid_argument!("token", "with payload"))?
                    .id
                    .clone();
                Ok(Self {
                    id,
                    claim: TokenClaim::Access(claim),
                    raw_parts: (signature.to_string(), message.to_string()),
                })
            }
        }
    }

    fn parse_raw(value: &str) -> Result<(Claim, &str, &str)> {
        let (signature, message) = expect_two(value.rsplitn(2, '.'))?;
        let (claim, _) = expect_two(message.rsplitn(2, '.'))?;
        let decode = base64::prelude::BASE64_URL_SAFE_NO_PAD
            .decode(claim)
            .map_err(|_| invalid_argument!("token", "base64 encode message"))?;
        let claim = serde_json::from_slice::<Claim>(decode.as_slice())
            .map_err(|_| invalid_argument!("token", "valid json encode claim"))?;
        Ok((claim, signature, message))
    }

    pub fn is_expired(&self) -> bool {
        let now = jsonwebtoken::get_current_timestamp();
        self.claim.inner().as_exp() < now - LEE_WAY
    }

    pub fn validate(
        &self,
        key: &jsonwebtoken::DecodingKey,
        algorithm: jsonwebtoken::Algorithm,
    ) -> Result<bool> {
        let signature = self.raw_parts.0.as_str();
        let message = self.raw_parts.1.as_str();
        let check = jsonwebtoken::crypto::verify(signature, message.as_ref(), key, algorithm)
            .map_err(|e| match e.into_kind() {
                ErrorKind::Base64(_) => invalid_argument!("token", "base64 encode signature"),
                _ => internal!(capture), // unreachable code reached, capture stacks trace and return
            })?;
        Ok(check)
    }

    pub fn validate_from_header(&self) -> Result<bool> {
        let (_, header) = expect_two(self.raw_parts.1.rsplitn(2, '.'))?;
        let decode = base64::prelude::BASE64_URL_SAFE_NO_PAD
            .decode(header)
            .map_err(|_| invalid_argument!("token", "base64 encode header"))?;
        let header = serde_json::from_slice::<jsonwebtoken::Header>(&decode)
            .map_err(|_| invalid_argument!("token", "json encode header"))?;
        let jwk = header
            .jwk
            .ok_or_else(|| invalid_argument!("token", "header with jwk"))?;
        let key = jsonwebtoken::DecodingKey::from_jwk(&jwk)
            .map_err(|_| invalid_argument!("token", "valid jwk in header"))?;
        let algorithm = jwk
            .common
            .algorithm
            .ok_or_else(|| invalid_argument!("token", "valid jwk with algorithm in header"))?;
        self.validate(&key, algorithm)
    }

    pub fn is_signed(&self) -> bool {
        !self.raw_parts.0.is_empty() && !self.raw_parts.1.is_empty()
    }

    pub fn sign(
        &mut self,
        key: &jsonwebtoken::EncodingKey,
        algorithm: jsonwebtoken::Algorithm,
    ) -> Result<String> {
        let header = jsonwebtoken::Header::new(algorithm);
        self._sign(key, &header)
    }

    fn _sign(
        &mut self,
        key: &jsonwebtoken::EncodingKey,
        header: &jsonwebtoken::Header,
    ) -> Result<String> {
        let signed = jsonwebtoken::encode(header, self.claim.inner(), key)
            .map_err(|_| internal!("cannot encode jwt with argument"))?;
        let (signature, message) = expect_two(signed.rsplitn(2, '.'))?;
        self.raw_parts = (signature.to_string(), message.to_string());
        Ok(signed)
    }

    pub fn sign_jwk(
        &mut self,
        key: &jsonwebtoken::EncodingKey,
        algorithm: jsonwebtoken::Algorithm,
        jwk: Jwk,
    ) -> Result<String> {
        let mut header = jsonwebtoken::Header::new(algorithm);
        header.jwk = Some(jwk);
        self._sign(key, &header)
    }

    pub fn to_pb(
        self,
        key: &jsonwebtoken::EncodingKey,
        algorithm: jsonwebtoken::Algorithm,
    ) -> Result<pb::Token> {
        if !self.is_signed() {
            return self.to_pb_signed(key, algorithm);
        }
        Ok(self.to_pb_exact())
    }

    pub fn to_pb_signed(
        mut self,
        key: &jsonwebtoken::EncodingKey,
        algorithm: jsonwebtoken::Algorithm,
    ) -> Result<pb::Token> {
        let kind = match self.claim {
            TokenClaim::Access(_) => pb::TokenKind::Access,
            TokenClaim::Refresh(_) => pb::TokenKind::Refresh,
        };
        Ok(pb::Token {
            value: self.sign(key, algorithm)?,
            kind: kind as i32,
        })
    }

    pub fn to_pb_exact(self) -> pb::Token {
        let kind = match self.claim {
            TokenClaim::Access(_) => pb::TokenKind::Access,
            TokenClaim::Refresh(_) => pb::TokenKind::Refresh,
        };
        let value = [self.raw_parts.1.as_str(), self.raw_parts.0.as_str()].join(".");
        pb::Token {
            value,
            kind: kind as i32,
        }
    }
}

#[test]
fn test() {
    let now = jsonwebtoken::get_current_timestamp();
    let mut token = Token::new(
        "1".into(),
        TokenKind::Access,
        Claim::builder(now + 176800)
            .audience("user")
            .subject("114514")
            .issue_at(now)
            .payload(Payload {
                id: "1".into(),
                kind: TokenKind::Access,
                detail: Some(pb::Payload {
                    group: "user".to_string(),
                    extra: "".to_string(),
                }),
            }),
    );
    let result = token.sign(
        &jsonwebtoken::EncodingKey::from_secret("1234567890".as_ref()),
        jsonwebtoken::Algorithm::HS256,
    );
    println!("{:?}", result);
    println!("{:?}", token);
    println!("{}", token.is_signed());
    println!("{}", token.is_expired());
    let ok = token.validate(
        &jsonwebtoken::DecodingKey::from_secret("1234567890".as_ref()),
        jsonwebtoken::Algorithm::HS256,
    );
    println!("{:?}", ok);
    let pb = token.to_pb_exact();
    println!("{:?}", pb);
}

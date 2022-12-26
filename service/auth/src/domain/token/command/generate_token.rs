use crate::domain::token::error::Error;
use crate::domain::token::model::store::{StoredToken, TokenStore};
use crate::domain::token::model::{Claim, Payload, Token, TokenClaim, TokenId};
use crate::domain::token::SERVICE_NAME;
use crate::domain::token::{RedisStore, Resolver};
use crate::pb;
use common::infra::{Args, Command};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::EncodingKey;
use std::collections::HashMap;
use std::time::Duration;

pub struct GenerateToken {
    pub id: String,  // sub | key id | token id
    pub sid: String, // aud | service id
    pub kind: pb::TokenKind,
    pub secret: Option<String>,
}

impl Args for GenerateToken {
    type Output = Result<pb::GenerateTokenResp, Error>;
}

async fn execute(
    req: GenerateToken,
    encode_key: &EncodingKey,
    clients: &HashMap<String, String>,
    pub_key_set: &JwkSet,
    exp_delta: &HashMap<String, i64>,
    refresh_delta_rate: i64,
    redis: &RedisStore,
) -> Result<pb::GenerateTokenResp, Error> {
    if let Some(secret) = req.secret.as_ref() {
        let check = clients.get(&req.sid).ok_or(Error::NoSID)?;
        if secret != check {
            return Err(Error::InvalidSecret(
                "secret does not match configuration".to_string(),
            ));
        }
    }

    // first check the cache
    if let Some(resp) = get_cache(&req.id, req.secret.is_some(), redis).await? {
        return Ok(resp);
    };

    // generate a new token pair
    let now_ts = chrono::Local::now().timestamp();
    let delta = *exp_delta.get(&*req.sid).ok_or(Error::NoSID)?;
    let mut access_token = Token {
        id: TokenId::from(&*req.id),
        claim: TokenClaim::Access(
            Claim::builder((now_ts + delta) as usize)
                .issue_at(now_ts as usize)
                .subject(&req.id)
                .audience(&req.sid)
                .issuer(SERVICE_NAME)
                .payload(Payload {
                    access: true,
                    secure: false,
                }),
        ),
    };
    let mut refresh_token = Token {
        id: TokenId::from(&*req.id),
        claim: TokenClaim::Refresh(
            Claim::builder((now_ts + refresh_delta_rate * delta) as usize)
                .issue_at(now_ts as usize)
                .subject(&req.id)
                .audience(&req.sid)
                .issuer(SERVICE_NAME)
                .payload(Payload {
                    access: false,
                    secure: false,
                }),
        ),
    };
    if req.kind == pb::TokenKind::Public {
        let access = access_token.public_token(pub_key_set, encode_key)?;
        let refresh = refresh_token.public_token(pub_key_set, encode_key)?;
        let token = StoredToken { access, refresh };
        // save to cache
        set_cache(
            &req.id,
            false,
            &token,
            Duration::from_secs(delta as u64),
            redis,
        )
        .await?;

        return Ok(pb::GenerateTokenResp {
            token: vec![token.access, token.refresh],
        });
    }

    let access = access_token.private_token(encode_key, req.secret.as_ref())?;
    let refresh = refresh_token.private_token(encode_key, req.secret.as_ref())?;
    let token = StoredToken { access, refresh };
    // save to cache
    set_cache(
        &req.id,
        req.secret.is_some(),
        &token,
        Duration::from_secs(delta as u64),
        redis,
    )
    .await?;

    Ok(pb::GenerateTokenResp {
        token: vec![token.access, token.refresh],
    })
}

async fn get_cache(
    id: &str,
    encrypt: bool,
    redis: &RedisStore,
) -> Result<Option<pb::GenerateTokenResp>, Error> {
    let token = redis.get_token(TokenId::from(id), encrypt).await?;
    if let Some(token) = token {
        return Ok(Some(pb::GenerateTokenResp {
            token: vec![token.access, token.refresh],
        }));
    }
    Ok(None)
}

async fn set_cache(
    id: &str,
    encrypt: bool,
    stored_token: &StoredToken,
    ttl: Duration,
    redis: &RedisStore,
) -> Result<(), Error> {
    redis
        .set_token(TokenId::from(id), encrypt, stored_token, ttl)
        .await?;
    Ok(())
}

impl Resolver {
    pub fn create_generate_token(&self) -> impl Command<GenerateToken> + '_ {
        move |req: GenerateToken| async move {
            let config = self.config();
            let redis = self.redis();
            let encode_key = if req.kind == pb::TokenKind::Public {
                self.encode_pem_key(&req.id)
                    .ok_or_else(|| Error::InvalidKid("cannot find this key".into()))?
            } else {
                self.encode_key()
            };
            execute(
                req,
                encode_key,
                &config.clients,
                &config.pub_jwks,
                &config.exp_delta,
                config.refresh_delta_rate,
                redis,
            )
            .await
        }
    }
}

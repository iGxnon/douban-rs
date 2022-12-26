use common::infra::*;
use jsonwebtoken::DecodingKey;
use std::collections::HashMap;

use crate::domain::token::error::Error;
use crate::domain::token::model::Token;
use crate::domain::token::Resolver;
use crate::{pb, pb::TokenKind};

pub struct AuthToken {
    pub token: pb::Token,
    pub secret: Option<String>,
}

impl Args for AuthToken {
    type Output = Result<pb::AuthTokenResp, Error>;
}

async fn execute(
    token: pb::Token,
    secret: Option<String>,
    decode_key: &DecodingKey,
    clients: &HashMap<String, String>,
) -> Result<pb::AuthTokenResp, Error> {
    if token.kind() == TokenKind::Public {
        let _ = Token::from_pub_token(&token)?;
        return Ok(pb::AuthTokenResp {});
    }

    // private token type
    let payload = Token::from_pri_token(&token, decode_key, secret.as_ref())?;
    if let Some(client_secret) = secret {
        let sid = payload.claim.inner().as_aud();
        let check = clients.get(sid).ok_or_else(|| {
            Error::InvalidToken(format!(
                "`aud`({}) should be a service(client) id, cannot found this sid in clients configurations",
                sid
            ))
        })?;
        if !client_secret.eq(check) {
            return Err(Error::InvalidToken(
                "service(client) not authorized, secret does not match configurations".into(),
            ));
        }
    }
    Ok(pb::AuthTokenResp {})
}

impl Resolver {
    pub fn create_auth_token(&self) -> impl Query<AuthToken> + '_ {
        move |req: AuthToken| async move {
            let config = self.config();
            execute(req.token, req.secret, self.decode_key(), &config.clients).await
        }
    }
}

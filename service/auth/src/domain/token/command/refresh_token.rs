use super::super::pb;
use crate::domain::token::model::Token;
use crate::domain::token::Resolver;
use common::infra::{Args, Command};
use common::invalid_argument;
use tonic::Status;

impl Args for pb::RefreshTokenReq {
    type Output = Result<pb::RefreshTokenRes, Status>;
}

async fn execute(
    req: pb::RefreshTokenReq,
    key: &jsonwebtoken::DecodingKey,
    algorithm: jsonwebtoken::Algorithm,
    generate_token: impl Command<pb::GenerateTokenReq>,
) -> Result<pb::RefreshTokenRes, Status> {
    let refresh_token = req.refresh.expect("token must be not none");
    let token = Token::from_pb(refresh_token)?;
    if token.is_expired() {
        return Err(invalid_argument!("refresh", "not expired token"));
    }
    if token.validate(key, algorithm)? {
        return Err(invalid_argument!("refresh"));
    }

    // generate new pair
    generate_token
        .execute(pb::GenerateTokenReq {
            sub: token.claim().as_sub().to_string(),
            aud: token.claim().as_aud().to_string(),
            jti: None,
            payload: token.payload().cloned().and_then(|payload| payload.detail),
        })
        .await
        .map(|token| pb::RefreshTokenRes {
            access: token.access,
            refresh: token.refresh,
        })
}

impl Resolver {
    pub fn create_refresh_token(&self) -> impl Command<pb::RefreshTokenReq> + '_ {
        move |req: pb::RefreshTokenReq| async move {
            let generate_token = self.create_generate_token();
            execute(req, &self.decode_key(), self.algorithm(), generate_token).await
        }
    }
}

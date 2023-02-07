use crate::domain::token::model::{Token, TokenKind};
use crate::domain::token::TokenResolver;
use common::infra::Command;
use common::invalid_argument;
use common::status::ext::GrpcResult;
use proto::pb::auth::token::v1 as pb;
use tracing::{instrument, trace};

#[instrument(skip_all, err)]
async fn execute(
    req: pb::RefreshTokenReq,
    key: &jsonwebtoken::DecodingKey,
    algorithm: jsonwebtoken::Algorithm,
    generate_token: impl Command<pb::GenerateTokenReq>,
) -> GrpcResult<pb::RefreshTokenRes> {
    let refresh_token = req.value.as_str();
    let token: Token = refresh_token.parse()?;
    trace!("Validating refresh token...");
    if token.kind() == TokenKind::Access {
        return Err(invalid_argument!("refresh", "refresh token kind").into());
    }
    if token.is_expired() {
        return Err(invalid_argument!("refresh", "not expired token").into());
    }
    if !token.validate(key, algorithm)? {
        return Err(invalid_argument!("refresh", "valid signature").into());
    }

    trace!("Refresh new token pair");
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

impl TokenResolver {
    pub fn create_refresh_token(&self) -> impl Command<pb::RefreshTokenReq> + '_ {
        move |req: pb::RefreshTokenReq| async move {
            let generate_token = self.create_generate_token();
            execute(req, self.decode_key(), self.algorithm(), generate_token).await
        }
    }
}

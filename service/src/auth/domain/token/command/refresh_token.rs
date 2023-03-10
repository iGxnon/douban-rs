use crate::auth::domain::token::model::token::{Token, TokenKind};
use crate::auth::rpc::TokenResolver;
use common::infra::Command;
use common::invalid_argument;
use common::status::ext::GrpcResult;
use proto::pb::auth::token::v1 as pb;
use tracing::instrument;

#[instrument(skip_all, err)]
async fn execute(
    req: pb::RefreshTokenReq,
    key: &jsonwebtoken::DecodingKey,
    algorithm: jsonwebtoken::Algorithm,
    generate_token: impl Command<pb::GenerateTokenReq>,
) -> GrpcResult<pb::RefreshTokenRes> {
    let refresh_token = req.value.as_str();
    let token: Token = refresh_token.parse()?;
    if token.kind() == TokenKind::Access {
        return Err(invalid_argument!("refresh", "refresh token kind").into());
    }
    let claim = token.claim()?;
    if claim.is_expired() {
        return Err(invalid_argument!("refresh", "not expired token").into());
    }
    if !token.validate(key, algorithm)? {
        return Err(invalid_argument!("refresh", "valid signature").into());
    }
    generate_token
        .execute(pb::GenerateTokenReq {
            sub: claim.as_sub().to_string(),
            aud: claim.as_aud().to_string(),
            jti: None,
            payload: claim.into_payload().and_then(|payload| payload.detail),
        })
        .await
        .map(|token| pb::RefreshTokenRes {
            access: token.access,
            refresh: token.refresh,
        })
}

impl TokenResolver {
    pub(in crate::auth) fn create_refresh_token(&self) -> impl Command<pb::RefreshTokenReq> + '_ {
        move |req: pb::RefreshTokenReq| async move {
            let generate_token = self.create_generate_token();
            execute(req, self.decode_key(), self.algorithm(), generate_token).await
        }
    }
}

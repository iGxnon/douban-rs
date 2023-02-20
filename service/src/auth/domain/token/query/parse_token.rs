use crate::auth::domain::token::model::token::{Token, TokenKind};
use crate::auth::domain::token::TokenResolver;
use common::infra::Query;
use common::status::ext::GrpcResult;
use proto::pb::auth::token::v1 as pb;
use tracing::instrument;
use tracing::log::trace;

#[instrument(skip_all, err)]
async fn execute(
    req: pb::ParseTokenReq,
    key: &jsonwebtoken::DecodingKey,
    algorithm: jsonwebtoken::Algorithm,
) -> GrpcResult<pb::ParseTokenRes> {
    trace!("Parsing token...");
    let token: Token = req.value.as_str().parse()?;
    let checked = token.validate(key, algorithm)?;
    let kind: pb::TokenKind = match token.kind() {
        TokenKind::Access => pb::TokenKind::Access,
        TokenKind::Refresh => pb::TokenKind::Refresh,
    };
    let claim = token.claim()?;
    Ok(pb::ParseTokenRes {
        checked,
        expired: claim.is_expired(),
        kind: kind as i32,
        payload: claim.into_payload().and_then(|payload| payload.detail),
    })
}

impl TokenResolver {
    pub fn create_parse_token(&self) -> impl Query<pb::ParseTokenReq> + '_ {
        move |req: pb::ParseTokenReq| async move {
            execute(req, self.decode_key(), self.algorithm()).await
        }
    }
}

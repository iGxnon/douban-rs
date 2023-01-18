use super::super::pb;
use crate::domain::token::model::Token;
use crate::domain::token::Resolver;
use common::infra::{Args, Query};
use tonic::Status;

impl Args for pb::ParseTokenReq {
    type Output = Result<pb::ParseTokenRes, Status>;
}

async fn execute(
    req: pb::ParseTokenReq,
    key: &jsonwebtoken::DecodingKey,
    algorithm: jsonwebtoken::Algorithm,
) -> Result<pb::ParseTokenRes, Status> {
    let token: Token = req.value.as_str().parse()?;
    let checked = token.validate(key, algorithm)?;
    let kind: pb::TokenKind = token.kind().into();
    Ok(pb::ParseTokenRes {
        checked,
        expired: token.is_expired(),
        kind: kind as i32,
        payload: token.payload().cloned().and_then(|payload| payload.detail),
    })
}

impl Resolver {
    pub fn create_parse_token(&self) -> impl Query<pb::ParseTokenReq> + '_ {
        move |req: pb::ParseTokenReq| async move {
            execute(req, &self.decode_key(), self.algorithm()).await
        }
    }
}

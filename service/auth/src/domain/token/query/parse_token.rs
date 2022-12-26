use jsonwebtoken::DecodingKey;

use common::infra::{Args, Query};

use crate::domain::token::error::Error;
use crate::domain::token::model::Token;
use crate::domain::token::Resolver;
use crate::pb;
use crate::pb::TokenKind;

pub struct ParseToken {
    pub token: pb::Token,
    pub secret: Option<String>,
}

impl Args for ParseToken {
    type Output = Result<pb::ParseTokenResp, Error>;
}

async fn execute(
    token: pb::Token,
    secret: Option<String>,
    decode_key: &DecodingKey,
) -> Result<pb::ParseTokenResp, Error> {
    let payload = if token.kind() == TokenKind::Public {
        let payload = Token::from_pub_token(&token)?.claim.into_inner();
        serde_json::to_string(&payload).map_err(Error::SerializerError)?
    } else {
        let payload = Token::from_pri_token(&token, decode_key, secret)?
            .claim
            .into_inner();
        serde_json::to_string(&payload).map_err(Error::SerializerError)?
    };
    Ok(pb::ParseTokenResp { payload })
}

impl Resolver {
    pub fn create_parse_token(&self) -> impl Query<ParseToken> + '_ {
        move |req: ParseToken| async { execute(req.token, req.secret, self.decode_key()).await }
    }
}

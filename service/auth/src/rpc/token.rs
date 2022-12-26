use crate::domain::token::command::generate_token::GenerateToken;
use crate::domain::token::error::ErrorExt;
use crate::domain::token::query::auth_token::AuthToken;
use crate::domain::token::query::parse_token::ParseToken;
use crate::domain::token::Resolver;
use crate::pb::token_srv_server::{TokenSrv, TokenSrvServer};
use crate::pb::{
    AuthTokenReq, AuthTokenResp, GenerateTokenReq, GenerateTokenResp, ParseTokenReq, ParseTokenResp,
};
use common::infra::*;
use std::ops::Deref;
use tonic::{Request, Response, Status};

pub struct TokenService(Resolver);

impl Deref for TokenService {
    type Target = Resolver;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TokenService {
    pub fn new(resolver: Resolver) -> Self {
        Self(resolver)
    }

    pub fn into_server(self) -> TokenSrvServer<TokenService> {
        TokenSrvServer::new(self)
    }
}

#[tonic::async_trait]
impl TokenSrv for TokenService {
    async fn generate_token(
        &self,
        req: Request<GenerateTokenReq>,
    ) -> Result<Response<GenerateTokenResp>, Status> {
        let req = req.into_inner();
        let generate_token = self.create_generate_token();
        generate_token
            .execute(GenerateToken {
                kind: req.kind(),
                id: req.id,
                sid: req.sid,
                secret: req.secret,
            })
            .await
            .into_response()
    }

    async fn auth_token(
        &self,
        req: Request<AuthTokenReq>,
    ) -> Result<Response<AuthTokenResp>, Status> {
        let req = req.into_inner();
        let auth_token = self.create_auth_token();
        auth_token
            .execute(AuthToken {
                token: req
                    .token
                    .ok_or_else(|| Status::invalid_argument("no token found"))?,
                secret: req.secret,
            })
            .await
            .into_response()
    }

    async fn parse_token(
        &self,
        req: Request<ParseTokenReq>,
    ) -> Result<Response<ParseTokenResp>, Status> {
        let req = req.into_inner();
        let parse_token = self.create_parse_token();
        parse_token
            .execute(ParseToken {
                token: req
                    .token
                    .ok_or_else(|| Status::invalid_argument("no token found"))?,
                secret: req.secret,
            })
            .await
            .into_response()
    }
}

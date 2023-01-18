use crate::domain::token::pb::{
    GenerateTokenReq, GenerateTokenRes, ParseTokenReq, ParseTokenRes, RefreshTokenReq,
    RefreshTokenRes,
};
use crate::domain::token::{pb, Resolver};
use common::infra::*;
use tonic::{async_trait, Request, Response, Status};

#[derive(Clone)]
pub struct TokenService(pub Resolver);

#[async_trait]
impl pb::token_service_server::TokenService for TokenService {
    async fn generate_token(
        &self,
        req: Request<GenerateTokenReq>,
    ) -> Result<Response<GenerateTokenRes>, Status> {
        let generate_token = self.0.create_generate_token();
        let res = generate_token.execute(req.into_inner()).await?;
        Ok(Response::new(res))
    }

    async fn parse_token(
        &self,
        req: Request<ParseTokenReq>,
    ) -> Result<Response<ParseTokenRes>, Status> {
        let parse_token = self.0.create_parse_token();
        let res = parse_token.execute(req.into_inner()).await?;
        Ok(Response::new(res))
    }

    async fn refresh_token(
        &self,
        req: Request<RefreshTokenReq>,
    ) -> Result<Response<RefreshTokenRes>, Status> {
        let refresh_token = self.0.create_refresh_token();
        let res = refresh_token.execute(req.into_inner()).await?;
        Ok(Response::new(res))
    }
}

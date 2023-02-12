use crate::auth::domain::token::TokenResolver;
use common::infra::*;
use proto::pb::auth::token::v1::*;
use tonic::{async_trait, Request, Response, Status};

#[derive(Clone)]
pub struct TokenService(pub TokenResolver);

#[async_trait]
impl token_service_server::TokenService for TokenService {
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

    async fn clear_cache(
        &self,
        req: Request<ClearCacheReq>,
    ) -> Result<Response<ClearCacheRes>, Status> {
        let clear_cache = self.0.create_clear_cache();
        let res = clear_cache.execute(req.into_inner()).await?;
        Ok(Response::new(res))
    }
}

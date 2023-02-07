use common::infra::Command;
use proto::pb::user::sys::v1::user_service_server;
use proto::pb::user::sys::v1::*;
use tonic::{Request, Response, Status};

use crate::domain::user::UserResolver;

pub struct UserService(pub(crate) UserResolver);

#[tonic::async_trait]
impl user_service_server::UserService for UserService {
    async fn login(&self, req: Request<LoginReq>) -> Result<Response<LoginRes>, Status> {
        let cmd = self.0.create_login();
        let resp = cmd.execute(req.into_inner()).await?;
        Ok(Response::new(resp))
    }

    async fn register(&self, req: Request<RegisterReq>) -> Result<Response<RegisterRes>, Status> {
        let cmd = self.0.create_register();
        let resp = cmd.execute(req.into_inner()).await?;
        Ok(Response::new(resp))
    }

    async fn bind(&self, req: Request<BindReq>) -> Result<Response<BindRes>, Status> {
        let cmd = self.0.create_bind();
        let resp = cmd.execute(req.into_inner()).await?;
        Ok(Response::new(resp))
    }
}

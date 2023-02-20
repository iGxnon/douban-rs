use crate::user::domain::user::model::user::User;
use common::{infra::Command, status::prelude::*};
use diesel::PgConnection;
use proto::pb::auth::token::v1::token_service_client::TokenServiceClient;
use proto::pb::user::sys::v1 as pb;
use std::ops::DerefMut;
use tonic::transport::Channel;

use crate::user::domain::user::UserResolver;

#[tracing::instrument(skip_all, err)]
async fn execute(
    req: pb::LoginReq,
    secret: &str,
    conn: &mut PgConnection,
    client: TokenServiceClient<Channel>,
) -> GrpcResult<pb::LoginRes> {
    let user = User::login(&req.identifier, &req.password, secret, conn)?;
    let result = user.sign_token_pair(client).await;
    result
}

impl UserResolver {
    pub fn create_login(&self) -> impl Command<pb::LoginReq> + '_ {
        move |req: pb::LoginReq| async move {
            execute(
                req,
                self.hash_secret(),
                self.pg_conn().deref_mut(),
                self.token_client(),
            )
            .await
        }
    }
}

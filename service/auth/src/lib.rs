use common::middleware::AuthBackend;
use futures::future::BoxFuture;

pub mod domain;
pub mod rpc;

pub mod pb {
    tonic::include_proto!("douban.auth.token");
}

pub use pb::token_srv_client;
pub use pb::token_srv_server;

// HttpAuth middleware AuthBackend implementation
//
// #[derive(Copy, Clone, Debug)]
// pub struct Backend;
//
// impl AuthBackend<UserId> for Backend {
//     fn auth_basic(&self, _username: &str, _password: &str) -> Result<UserId, String> {
//         Ok(UserId::new_u64())
//     }
//
//     fn auth_basic_async(
//         &self,
//         _username: &str,
//         _password: &str,
//     ) -> BoxFuture<'static, Result<(UserId, Option<String>), String>> {
//         Box::pin(async { Ok((UserId::new_u64(), None)) })
//     }
//
//     fn auth_bearer(
//         &self,
//         _token: &str,
//     ) -> BoxFuture<'static, Result<(UserId, Option<String>), String>> {
//         Box::pin(async { Ok((UserId::new_u64(), None)) })
//     }
// }

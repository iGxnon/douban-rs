use common::middleware::AuthBackend;
use common::model::UserId;
use common::model::UserId::UserIdU64;
use futures::future::BoxFuture;

// HttpAuth middleware AuthBackend implementation

#[derive(Copy, Clone, Debug)]
pub struct Backend;

impl AuthBackend<UserId> for Backend {
    fn auth_basic(&self, _username: &str, _password: &str) -> Result<UserId, String> {
        Ok(UserIdU64(0))
    }

    fn auth_basic_async(
        &self,
        _username: &str,
        _password: &str,
    ) -> BoxFuture<'static, Result<(UserId, Option<String>), String>> {
        Box::pin(async { Ok((UserIdU64(0), None)) })
    }

    fn auth_bearer(
        &self,
        _token: &str,
    ) -> BoxFuture<'static, Result<(UserId, Option<String>), String>> {
        Box::pin(async { Ok((UserIdU64(0), None)) })
    }
}

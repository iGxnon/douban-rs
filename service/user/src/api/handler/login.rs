use super::*;

pub(in crate::api) async fn handle(
    State(resolver): State<Arc<Resolver>>,
    Form(SharedLoginReq { username, password }): Form<SharedLoginReq>,
) -> error::Result<()> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    Err(Error::WrongPassword)
}

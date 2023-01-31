use super::*;

async fn execute<'a>() -> HttpResult<'a, ()> {
    Ok(None)
}

pub(in crate::rest) async fn handle<'a>(
    State(resolver): State<Arc<RestResolver>>,
    Form(SharedLoginReq { username, password }): Form<SharedLoginReq>,
) -> Json<Resp<'a, ()>> {
    Json(execute::<'a>().await.into())
}

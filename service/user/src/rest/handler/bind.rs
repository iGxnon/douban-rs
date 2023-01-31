use super::*;

async fn execute<'a>() -> HttpResult<'a, ()> {
    Ok(None)
}

pub(in crate::rest) async fn handle<'a>(
    State(resolver): State<Arc<RestResolver>>,
    Form(BindReq {
        email,
        phone,
        oauth_id,
    }): Form<BindReq>,
) -> Json<Resp<'a, ()>> {
    Json(execute::<'a>().await.into())
}

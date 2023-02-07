use super::*;

pub(in crate::rest) async fn handle(
    State(resolver): State<Arc<RestResolver>>,
    Form(RegisterReq { username, password }): Form<RegisterReq>,
) -> Json<Resp<()>> {
    let resp = resolver
        .user_service()
        .register(pb::RegisterReq { username, password })
        .await
        .map(|_| ())
        .map_err(HttpStatus::from);

    Json(resp.into())
}

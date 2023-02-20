use super::*;

pub(crate) async fn handle(
    State(resolver): State<Arc<RestResolver>>,
    Form(RegisterReq { username, password }): Form<RegisterReq>,
) -> (StatusCode, Json<Resp<()>>) {
    let resp = resolver
        .user_service()
        .register(pb::RegisterReq { username, password })
        .await
        .map(|_| ())
        .map_err(HttpStatus::from);

    (resp.http_code(), Json(resp.into()))
}

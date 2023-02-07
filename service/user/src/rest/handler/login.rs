use super::*;

pub(in crate::rest) async fn handle(
    State(resolver): State<Arc<RestResolver>>,
    Form(LoginReq {
        identifier,
        password,
    }): Form<LoginReq>,
) -> Json<Resp<pb::LoginRes>> {
    let resp = resolver
        .user_service()
        .login(pb::LoginReq {
            identifier,
            password,
        })
        .await
        .map(|res| res.into_inner())
        .map_err(HttpStatus::from);

    Json(resp.into())
}

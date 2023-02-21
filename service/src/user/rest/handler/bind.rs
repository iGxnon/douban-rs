use super::*;

pub(crate) async fn handle(
    State(resolver): State<Arc<RestResolver>>,
    uid: Extension<UserId>,
    Form(BindReq {
        email,
        phone,
        github,
    }): Form<BindReq>,
) -> (StatusCode, Json<Resp<()>>) {
    let resp = resolver
        .user_client()
        .bind(pb::BindReq {
            identifier: uid.as_string(),
            email,
            phone,
            github: github.map(|v| v.to_string()),
        })
        .await
        .map(|_| ())
        .map_err(HttpStatus::from);
    (resp.http_code(), Json(resp.into()))
}

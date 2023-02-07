use super::*;

pub(in crate::rest) async fn handle(
    State(resolver): State<Arc<RestResolver>>,
    Form(BindReq {
        email,
        phone,
        oauth_id,
    }): Form<BindReq>,
) -> Json<Resp<()>> {
    let resp = resolver
        .user_service()
        .bind(pb::BindReq {
            email,
            phone,
            oauth_id,
        })
        .await
        .map(|_| ())
        .map_err(HttpStatus::from);

    Json(resp.into())
}

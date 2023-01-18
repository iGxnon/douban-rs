use super::*;

pub(in crate::api) async fn handle(
    State(resolver): State<Arc<Resolver>>,
    Form(BindReq {
        email,
        phone,
        oauth_id,
    }): Form<BindReq>,
) -> error::Result<()> {
    todo!()
}

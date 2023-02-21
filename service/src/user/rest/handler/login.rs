use super::*;
use cookie::{Cookie, CookieJar};
use cookie::time::Duration;
use http::HeaderMap;
use common::layer::write_cookie;
use crate::auth::layer::KEY;

pub(crate) async fn handle(
    State(resolver): State<Arc<RestResolver>>,
    Form(LoginReq {
        identifier,
        password,
    }): Form<LoginReq>,
) -> (StatusCode, HeaderMap, Json<Resp<pb::LoginRes>>) {
    let resp = resolver
        .user_client()
        .login(pb::LoginReq {
            identifier,
            password,
        })
        .await
        .map(|res| res.into_inner())
        .map_err(HttpStatus::from);
    let mut header_map = HeaderMap::default();
    let mut cookie_jar = CookieJar::new();
    if let Ok(ref res) = resp {
        let conf = resolver.conf.cookie_conf.clone();
        let cookie = Cookie::build(
            conf.cookie_name,
            format!(
                "{}|{}",
                res.access.clone().expect("Gateway error").value, // None indicates that something wrong with gateway
                res.refresh.clone().expect("Gateway error").value
            ),
        )
        .domain(conf.domain)
        .max_age(Duration::seconds(conf.max_age))
        .same_site(conf.same_site.into())
        .path(conf.path)
        .http_only(conf.http_only)
        .secure(conf.secure)
        .finish();
        if let Some(key) = KEY.get() {
            cookie_jar.private_mut(key).add(cookie);
        } else {
            cookie_jar.add(cookie);
        }
    }
    write_cookie(&mut header_map, &cookie_jar);
    (resp.http_code(), header_map, Json(resp.into()))
}

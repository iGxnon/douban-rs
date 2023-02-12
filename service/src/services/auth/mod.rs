pub mod domain;
pub mod rpc;

pub mod layer {
    use super::domain::token::TokenResolver;
    use common::infra::Resolver;
    use common::layer::{AsyncAuth, CookieAuthConf, DEFAULT_COOKIE_NAME};
    use common::middleware::etcd::EtcdConf;
    use common::registry::{EtcdRegistry, ServiceDiscover};
    use cookie::Key;
    use futures::future::BoxFuture;
    use http::{HeaderMap, Request, Response};
    use once_cell::sync::OnceCell;
    use proto::pb::auth::token::v1::token_service_client::TokenServiceClient;
    use serde::{Deserialize, Serialize};
    use std::future::Future;
    use std::marker::PhantomData;
    use std::sync::Arc;
    use tonic::transport::Channel;

    static CLIENT: OnceCell<TokenServiceClient<Channel>> = OnceCell::new();
    static KEY: OnceCell<Key> = OnceCell::new();

    const DEFAULT_REALM: &str = "app-http-auth";

    fn default_etcd() -> Option<EtcdConf> {
        Some(EtcdConf::default())
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct AuthConf {
        #[serde(default = "default_etcd")]
        pub etcd: Option<EtcdConf>,
        #[serde(default)]
        pub cookie_conf: Option<CookieAuthConf>,
        #[serde(default)]
        pub bearer_header: Option<String>,
        #[serde(default)]
        pub www_auth: WWWAuth,
    }

    pub trait IdentityProvider: Clone {
        // Id, Group, Extra are used to identify whom the token refer to.
        type Id: From<String> + Send + Sync + 'static;
        type Group: From<String> + Send + Sync + 'static;
        // Extra might deserialize extra string into some other type,
        // so I used TryFrom here.
        type Extra: TryFrom<String> + Send + Sync + 'static;
    }

    impl Default for AuthConf {
        fn default() -> Self {
            Self {
                etcd: default_etcd(),
                cookie_conf: Some(CookieAuthConf::default()),
                bearer_header: Some(http::header::AUTHORIZATION.to_string()),
                www_auth: Default::default(),
            }
        }
    }

    /// Used for generating unauthorized header [WWW_AUTH_BASIC], [WWW_AUTH_BEARER], [WWW_AUTH_COOKIE]
    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct WWWAuth {
        #[serde(default)]
        bearer: Option<String>,
        #[serde(default)]
        cookie: Option<String>,
    }

    impl WWWAuth {
        pub fn bearer(realm: &str) -> Self {
            Self {
                bearer: Some(format!(r#"Bearer realm={},charset=UTF-8"#, realm)),
                cookie: None,
            }
        }

        pub fn cookie(realm: &str, cookie_name: &str) -> Self {
            Self {
                bearer: None,
                cookie: Some(format!(
                    r#"Cookie realm={},charset=UTF-8,cookie-name={}"#,
                    realm, cookie_name
                )),
            }
        }
    }

    impl Default for WWWAuth {
        fn default() -> Self {
            Self {
                bearer: Some(format!(r#"Bearer realm={},charset=UTF-8"#, DEFAULT_REALM)),
                cookie: Some(format!(
                    r#"Cookie realm={},charset=UTF-8,cookie-name={}"#,
                    DEFAULT_REALM, DEFAULT_COOKIE_NAME,
                )),
            }
        }
    }

    pub struct Auth<I: IdentityProvider, ResBody> {
        method: method::Method,
        conf: AuthConf,
        _data: PhantomData<(ResBody, I)>,
    }

    impl<I: IdentityProvider, ResBody> Clone for Auth<I, ResBody> {
        fn clone(&self) -> Self {
            Self {
                method: self.method.clone(),
                conf: self.conf.clone(),
                _data: Default::default(),
            }
        }
    }

    pub trait AuthMethod<B, ResBody, I: IdentityProvider>: Clone {
        type ReqBody;
        type Fut: Future<
            Output = Result<(Request<Self::ReqBody>, Option<HeaderMap>), Response<ResBody>>,
        >;
        fn auth(
            &self,
            client: TokenServiceClient<Channel>,
            cookie_conf: AuthConf,
            req: Request<B>,
        ) -> Self::Fut;
    }

    impl<ResBody, I: IdentityProvider> Auth<I, ResBody> {
        pub async fn cookie(mut conf: AuthConf) -> Self {
            // reduce clone cost
            conf.www_auth.bearer.take();
            conf.bearer_header.take();
            debug_assert!(conf.cookie_conf.is_some());
            Self {
                method: method::Method::CookieAuth(Arc::new(
                    conf.www_auth
                        .cookie
                        .take()
                        .expect("Require www_auth.cookie to be represented"),
                )),
                conf,
                _data: PhantomData,
            }
            .connect()
            .await
        }

        pub async fn bearer(mut conf: AuthConf) -> Self {
            // reduce clone cost
            conf.www_auth.cookie.take();
            conf.cookie_conf.take();
            debug_assert!(conf.bearer_header.is_some());
            Self {
                method: method::Method::BearerAuth(Arc::new(
                    conf.www_auth
                        .bearer
                        .take()
                        .expect("Require www_auth.bearer to be represented"),
                )),
                conf,
                _data: PhantomData,
            }
            .connect()
            .await
        }

        async fn connect(mut self) -> Self {
            let (channel, tx) = Channel::balance_channel(64);
            let discover = EtcdRegistry::discover(self.conf.etcd.take().unwrap());
            discover
                .discover_to_channel(TokenResolver::DOMAIN, tx)
                .await
                .expect("Cannot connect to auth service_old.");
            CLIENT
                .try_insert(TokenServiceClient::new(channel))
                .expect("Cannot connect twice.");
            self
        }
    }

    type BoxFut<B, ResBody> =
        BoxFuture<'static, Result<(Request<B>, Option<HeaderMap>), Response<ResBody>>>;

    impl<B, ResBody, I> AsyncAuth<B> for Auth<I, ResBody>
    where
        B: Send + 'static,
        I: IdentityProvider,
        ResBody: Default + Send,
    {
        type RequestBody = B;
        type ResponseBody = ResBody;
        type Future = BoxFut<B, ResBody>;

        fn authorize(&mut self, req: Request<B>) -> Self::Future {
            let method = &self.method;
            let client = CLIENT.get().expect("Not connect.").clone();

            method.auth::<I, _, _>(client, self.conf.clone(), req)
        }
    }

    mod method {
        use super::*;
        use common::layer::{scan_cookies, write_cookie};
        use common::status::prelude::*;
        use cookie::time::Duration;
        use cookie::{Cookie, CookieJar};
        use http::header::HeaderName;
        use http::{header, Request, Response, StatusCode};
        use proto::pb::auth::token::v1 as pb;
        use std::sync::Arc;
        use tracing::instrument;
        use tracing::log::trace;

        #[derive(Clone)]
        pub enum Method {
            CookieAuth(Arc<String>),
            BearerAuth(Arc<String>),
        }

        fn expect_two<I: Iterator>(mut split: I) -> Result<(I::Item, I::Item), StatusCode> {
            match (split.next(), split.next(), split.next()) {
                (Some(one), Some(two), None) => Ok((one, two)),
                _ => Err(StatusCode::UNAUTHORIZED),
            }
        }

        impl Method {
            async fn cookie_auth<I: IdentityProvider, B>(
                mut client: TokenServiceClient<Channel>,
                conf: CookieAuthConf,
                mut req: Request<B>,
            ) -> Result<(Request<B>, Option<CookieJar>), StatusCode> {
                let mut cookie_jar = scan_cookies(&req);
                let cookie = match conf.encrypted {
                    None => cookie_jar
                        .get(&conf.cookie_name)
                        .ok_or(StatusCode::UNAUTHORIZED)?
                        .clone(),
                    Some(key) => {
                        let key = KEY.get_or_init(|| Key::derive_from(key.as_bytes()));
                        cookie_jar
                            .private(key)
                            .get(&conf.cookie_name)
                            .ok_or(StatusCode::UNAUTHORIZED)?
                    }
                };

                let (access, refresh) = expect_two(cookie.value().split('|'))?;

                let res = client
                    .parse_token(pb::ParseTokenReq {
                        value: access.to_string(),
                    })
                    .await
                    .map_err(|e| e.code().to_http_code())?
                    .into_inner();

                if res.kind() != pb::TokenKind::Access {
                    return Err(StatusCode::UNAUTHORIZED);
                }

                trace!("Check if it is valid");
                if !res.checked {
                    return Err(StatusCode::UNAUTHORIZED);
                }

                trace!("Check if it is expired");
                if res.expired {
                    trace!("Try to refresh token.");
                    let refresh_res = client
                        .refresh_token(pb::RefreshTokenReq {
                            value: refresh.to_string(),
                        })
                        .await
                        .map_err(|e| e.code().to_http_code())?
                        .into_inner();

                    let cookie = Cookie::build(
                        conf.cookie_name,
                        format!(
                            "{}|{}",
                            refresh_res.access.ok_or(StatusCode::BAD_GATEWAY)?.value, // None indicates that something wrong with gateway
                            refresh_res.refresh.ok_or(StatusCode::BAD_GATEWAY)?.value
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
                    trace!("Refreshed token.");

                    if let Some(payload) = res.payload {
                        trace!("Write sub, group, extra into request.");
                        let extensions = req.extensions_mut();
                        extensions.insert(I::Id::from(payload.sub));
                        extensions.insert(I::Group::from(payload.group));
                        extensions.insert(
                            I::Extra::try_from(payload.extra)
                                .map_err(|_| StatusCode::BAD_GATEWAY)?,
                        );
                    }

                    return Ok((req, Some(cookie_jar)));
                }

                if let Some(payload) = res.payload {
                    trace!("Write sub, group, extra into request.");
                    let extensions = req.extensions_mut();
                    extensions.insert(I::Id::from(payload.sub));
                    extensions.insert(I::Group::from(payload.group));
                    extensions.insert(
                        I::Extra::try_from(payload.extra).map_err(|_| StatusCode::BAD_GATEWAY)?,
                    );
                }

                Ok((req, None))
            }

            #[instrument(err, skip_all)]
            async fn bearer_auth<I: IdentityProvider, B>(
                mut client: TokenServiceClient<Channel>,
                bearer_header: HeaderName,
                mut req: Request<B>,
            ) -> Result<(Request<B>, Option<HeaderMap>), StatusCode> {
                let token = req
                    .headers()
                    .get(&bearer_header)
                    .ok_or(StatusCode::UNAUTHORIZED)?
                    .to_str()
                    .map_err(|_| StatusCode::UNAUTHORIZED)?
                    .trim_start_matches("Bearer ");

                let res = client
                    .parse_token(pb::ParseTokenReq {
                        value: token.to_string(),
                    })
                    .await
                    .map_err(|e| e.code().to_http_code())?
                    .into_inner();

                trace!("Check if it is expired.");
                if res.expired {
                    return Err(StatusCode::UNAUTHORIZED);
                }

                trace!("Check if it is valid.");
                if !res.checked {
                    return Err(StatusCode::UNAUTHORIZED);
                }

                trace!("Check if need to refresh.");
                if res.kind() == pb::TokenKind::Refresh {
                    trace!("Try to refresh token.");
                    let refresh_res = client
                        .refresh_token(pb::RefreshTokenReq {
                            value: token.to_string(),
                        })
                        .await
                        .map_err(|e| e.code().to_http_code())?
                        .into_inner();

                    let mut header_map = HeaderMap::new();

                    let set_token = format!(
                        "access={},refresh={}",
                        refresh_res.access.ok_or(StatusCode::BAD_GATEWAY)?.value,
                        refresh_res.refresh.ok_or(StatusCode::BAD_GATEWAY)?.value,
                    )
                    .parse()
                    .map_err(|_| StatusCode::BAD_GATEWAY)?;

                    let set_token_header = format!("set-{}", bearer_header.as_str().to_lowercase());

                    header_map.insert(
                        HeaderName::from_lowercase(set_token_header.as_bytes())
                            .expect("Config auth_header is not valid."),
                        set_token,
                    );

                    trace!(
                        "Refreshed token and set to request header '{}'.",
                        set_token_header
                    );

                    if let Some(payload) = res.payload {
                        trace!("Write sub, group, extra into request.");
                        let extensions = req.extensions_mut();
                        extensions.insert(I::Id::from(payload.sub));
                        extensions.insert(I::Group::from(payload.group));
                        extensions.insert(
                            I::Extra::try_from(payload.extra)
                                .map_err(|_| StatusCode::BAD_GATEWAY)?,
                        );
                    }

                    return Ok((req, Some(header_map)));
                }

                if let Some(payload) = res.payload {
                    trace!("Write sub, group, extra into request.");
                    let extensions = req.extensions_mut();
                    extensions.insert(I::Id::from(payload.sub));
                    extensions.insert(I::Group::from(payload.group));
                    extensions.insert(
                        I::Extra::try_from(payload.extra).map_err(|_| StatusCode::BAD_GATEWAY)?,
                    );
                }

                Ok((req, None))
            }
        }

        impl Method {
            pub(super) fn auth<I: IdentityProvider, B: Send + 'static, ResBody: Default + Send>(
                &self,
                client: TokenServiceClient<Channel>,
                conf: AuthConf,
                req: Request<B>,
            ) -> BoxFut<B, ResBody> {
                let method = self.clone();
                Box::pin(async move {
                    let res = match method {
                        Method::CookieAuth(_) => Method::cookie_auth::<I, _>(
                            client,
                            conf.cookie_conf.expect("Must configure cookie_conf"),
                            req,
                        )
                        .await
                        .map(|(req, cookies)| match cookies {
                            None => (req, None),
                            Some(cookie_jar) => {
                                let mut header_map = HeaderMap::new();
                                write_cookie(&mut header_map, &cookie_jar);
                                (req, Some(header_map))
                            }
                        }),
                        Method::BearerAuth(_) => {
                            Method::bearer_auth::<I, _>(
                                client,
                                conf.bearer_header
                                    .expect("Must configure bearer_header")
                                    .as_str()
                                    .parse()
                                    .expect("Not a valid bearer http header"),
                                req,
                            )
                            .await
                        }
                    };
                    match res {
                        Ok(req) => Ok(req),
                        Err(code) => match method {
                            Method::CookieAuth(www) => Err(Response::builder()
                                .status(code)
                                .header(header::WWW_AUTHENTICATE, www.as_str())
                                .body(ResBody::default())
                                .unwrap()),
                            Method::BearerAuth(www) => Err(Response::builder()
                                .status(code)
                                .header(header::WWW_AUTHENTICATE, www.as_str())
                                .body(ResBody::default())
                                .unwrap()),
                        },
                    }
                })
            }
        }
    }
}

pub mod domain;
pub mod rpc;

pub mod layer {
    use crate::domain::token::TokenResolver;
    use common::discover::{Discover, EtcdDiscover, EtcdDiscoverConf};
    use common::infra::Resolver;
    use common::layer::{AsyncAuth, CookieAuthConf, DEFAULT_COOKIE_NAME};
    use cookie::Key;
    use http::{HeaderMap, Request, Response};
    use once_cell::sync::OnceCell;
    use proto::pb::auth::token::v1::token_service_client::TokenServiceClient;
    use serde::{Deserialize, Serialize};
    use std::future::Future;
    use std::marker::PhantomData;
    use tonic::transport::Channel;

    static CLIENT: OnceCell<TokenServiceClient<Channel>> = OnceCell::new();
    static KEY: OnceCell<Key> = OnceCell::new();

    const DEFAULT_REALM: &str = "app-http-auth";

    #[derive(Deserialize, Serialize, Clone, Debug)]
    #[serde(default)]
    pub struct AuthConf {
        pub auth_discover: EtcdDiscoverConf,
        pub cookie_conf: Option<CookieAuthConf>,
        pub auth_header: String,
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
                auth_discover: Default::default(),
                cookie_conf: Some(CookieAuthConf::default()),
                auth_header: http::header::AUTHORIZATION.to_string(),
                www_auth: Default::default(),
            }
        }
    }

    /// Used for generating unauthorized header [WWW_AUTH_BASIC], [WWW_AUTH_BEARER], [WWW_AUTH_COOKIE]
    #[derive(Deserialize, Serialize, Clone, Debug)]
    #[serde(default)]
    pub struct WWWAuth {
        basic: String,
        bearer: String,
        cookie: String,
    }

    impl WWWAuth {
        pub fn new(realm: &str, cookie_name: &str) -> Self {
            Self {
                basic: format!(r#"Basic realm={},charset=UTF-8"#, realm),
                bearer: format!(r#"Bearer realm={},charset=UTF-8"#, realm),
                cookie: format!(
                    r#"Cookie realm={},charset=UTF-8,cookie-name={}"#,
                    realm, cookie_name
                ),
            }
        }
    }

    impl Default for WWWAuth {
        fn default() -> Self {
            Self {
                basic: format!(r#"Basic realm={},charset=UTF-8"#, DEFAULT_REALM),
                bearer: format!(r#"Bearer realm={},charset=UTF-8"#, DEFAULT_REALM),
                cookie: format!(
                    r#"Cookie realm={},charset=UTF-8,cookie-name={}"#,
                    DEFAULT_REALM, DEFAULT_COOKIE_NAME,
                ),
            }
        }
    }

    pub struct Auth<I: IdentityProvider, ResBody, F: Clone> {
        f: F,
        conf: AuthConf,
        _data: PhantomData<(ResBody, I)>,
    }

    impl<I: IdentityProvider, ResBody, F: Clone> Clone for Auth<I, ResBody, F> {
        fn clone(&self) -> Self {
            Self {
                f: self.f.clone(),
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

    impl<ResBody, I: IdentityProvider> Auth<I, ResBody, method::Method> {
        pub async fn cookie(conf: AuthConf) -> Self {
            Self {
                f: method::Method::CookieAuth,
                conf,
                _data: PhantomData,
            }
            .connect()
            .await
        }

        pub async fn bearer(conf: AuthConf) -> Self {
            Self {
                f: method::Method::BearerJwtAuth,
                conf,
                _data: PhantomData,
            }
            .connect()
            .await
        }

        async fn connect(self) -> Self {
            let (channel, tx) = Channel::balance_channel(64);
            let discover = EtcdDiscover::new(self.conf.auth_discover.clone());
            discover
                .discover_to_channel(TokenResolver::DOMAIN, tx)
                .await
                .expect("Cannot connect to auth service.");
            CLIENT
                .try_insert(TokenServiceClient::new(channel))
                .expect("Cannot connect twice.");
            self
        }
    }

    impl<B, F, ResBody, I> AsyncAuth<B> for Auth<I, ResBody, F>
    where
        B: Send + 'static,
        F: AuthMethod<B, ResBody, I>,
        I: IdentityProvider,
    {
        type RequestBody = F::ReqBody;
        type ResponseBody = ResBody;
        type Future = F::Fut;

        fn authorize(&mut self, req: Request<B>) -> Self::Future {
            let method = &self.f;
            let client = CLIENT.get().expect("Not connect.").clone();

            method.auth(client, self.conf.clone(), req)
        }
    }

    pub mod method {
        use super::*;
        use common::layer::{scan_cookies, write_cookie};
        use common::status::prelude::*;
        use cookie::time::Duration;
        use cookie::{Cookie, CookieJar};
        use futures::future::BoxFuture;
        use http::header::HeaderName;
        use http::{header, Request, Response, StatusCode};
        use proto::pb::auth::token::v1 as pb;
        use tracing::instrument;
        use tracing::log::trace;

        #[derive(Clone)]
        pub enum Method {
            CookieAuth,
            BearerJwtAuth,
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

        impl<B, ResBody, I> AuthMethod<B, ResBody, I> for Method
        where
            B: Send + 'static,
            ResBody: Send + Default,
            I: IdentityProvider,
        {
            type ReqBody = B;
            type Fut = BoxFuture<
                'static,
                Result<(Request<Self::ReqBody>, Option<HeaderMap>), Response<ResBody>>,
            >;

            fn auth(
                &self,
                client: TokenServiceClient<Channel>,
                conf: AuthConf,
                req: Request<B>,
            ) -> Self::Fut {
                let method = self.clone();
                Box::pin(async move {
                    let res = match method {
                        Method::CookieAuth => Method::cookie_auth::<I, _>(
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
                        Method::BearerJwtAuth => {
                            Method::bearer_auth::<I, _>(
                                client,
                                conf.auth_header
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
                            Method::CookieAuth => Err(Response::builder()
                                .status(code)
                                .header(header::WWW_AUTHENTICATE, conf.www_auth.cookie)
                                .body(ResBody::default())
                                .unwrap()),
                            Method::BearerJwtAuth => Err(Response::builder()
                                .status(code)
                                .header(header::WWW_AUTHENTICATE, conf.www_auth.bearer)
                                .body(ResBody::default())
                                .unwrap()),
                        },
                    }
                })
            }
        }
    }
}

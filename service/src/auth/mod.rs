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
    use std::marker::PhantomData;
    use std::sync::Arc;
    use tonic::transport::Channel;

    static CLIENT: OnceCell<TokenServiceClient<Channel>> = OnceCell::new();
    pub static KEY: OnceCell<Key> = OnceCell::new();

    const DEFAULT_REALM: &str = "app-http-auth";

    pub trait IdentityProvider: Clone {
        // Id, Group, Extra are used to identify whom the token refer to.
        type Id: From<String> + Send + Sync + 'static;
        type Group: From<String> + Send + Sync + 'static;
        // Extra might deserialize extra string into some other type,
        // so I used TryFrom here.
        type Extra: TryFrom<String> + Send + Sync + 'static;
    }

    /// Used for generating unauthorized header [WWW_AUTH_BASIC], [WWW_AUTH_BEARER], [WWW_AUTH_COOKIE]
    pub struct WWWAuth(String);

    impl WWWAuth {
        pub fn bearer(realm: &str) -> Self {
            Self(format!(r#"Bearer realm={},charset=UTF-8"#, realm))
        }

        pub fn cookie(realm: &str, cookie_name: &str) -> Self {
            Self(format!(
                r#"Cookie realm={},charset=UTF-8,cookie-name={}"#,
                realm, cookie_name
            ))
        }
    }

    pub struct AuthBuilder {
        etcd: EtcdConf,
        method: method::Method,
    }

    impl AuthBuilder {
        pub fn cookie(etcd: EtcdConf) -> Self {
            Self {
                etcd,
                method: method::Method::CookieAuth {
                    www: Arc::new(WWWAuth::cookie(DEFAULT_REALM, DEFAULT_COOKIE_NAME).0),
                    auth_conf: Default::default(),
                },
            }
        }

        pub fn bearer(etcd: EtcdConf) -> Self {
            Self {
                etcd,
                method: method::Method::BearerAuth {
                    www: Arc::new(WWWAuth::bearer(DEFAULT_REALM).0),
                    header: Arc::new("".to_string()),
                },
            }
        }

        pub fn cookie_conf(mut self, conf: CookieAuthConf) -> Self {
            if let method::Method::CookieAuth { auth_conf, .. } = &mut self.method {
                *auth_conf = conf;
            }
            self
        }

        pub fn bearer_header(mut self, bearer_header: &str) -> Self {
            if let method::Method::BearerAuth { header, .. } = &mut self.method {
                *header = Arc::new(bearer_header.to_string());
            }
            self
        }

        pub fn www(mut self, www_auth: WWWAuth) -> Self {
            if let method::Method::CookieAuth { www, .. } = &mut self.method {
                *www = Arc::new(www_auth.0.clone());
            }
            self
        }

        pub fn www_str(mut self, www_auth: &str) -> Self {
            if let method::Method::CookieAuth { www, .. } = &mut self.method {
                *www = Arc::new(www_auth.to_string());
            }
            self
        }

        pub async fn finish<I: IdentityProvider, ResBody>(self) -> Auth<I, ResBody> {
            let (channel, tx) = Channel::balance_channel(64);
            let discover = EtcdRegistry::discover(self.etcd);
            let service_key = TokenResolver::service_key();
            discover
                .discover_to_channel(&service_key, tx)
                .await
                .expect("Cannot connect to auth service.");
            CLIENT
                .try_insert(TokenServiceClient::new(channel))
                .expect("Cannot connect twice.");
            if let method::Method::CookieAuth { ref auth_conf, .. } = self.method {
                if let Some(ref key) = auth_conf.encrypted {
                    KEY.try_insert(Key::derive_from(key.as_bytes()))
                        .unwrap_or_else(|_| panic!("Cannot create cookie key"));
                }
            }
            Auth {
                method: self.method,
                _data: Default::default(),
            }
        }
    }

    pub struct Auth<I: IdentityProvider, ResBody> {
        method: method::Method,
        _data: PhantomData<(ResBody, I)>,
    }

    impl<I: IdentityProvider, ResBody> Clone for Auth<I, ResBody> {
        fn clone(&self) -> Self {
            Self {
                method: self.method.clone(),
                _data: Default::default(),
            }
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
            let method = self.method.clone();
            let client = CLIENT.get().expect("Not connect.").clone();

            method.auth::<I, _, _>(client, req)
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
        use proto::pb::auth::token::v1::Payload;
        use std::sync::Arc;
        use tracing::log::trace;

        #[derive(Clone)]
        pub enum Method {
            CookieAuth {
                www: Arc<String>,
                auth_conf: CookieAuthConf,
            },
            BearerAuth {
                www: Arc<String>,
                header: Arc<String>,
            },
        }

        fn expect_two<I: Iterator>(mut split: I) -> Result<(I::Item, I::Item), StatusCode> {
            match (split.next(), split.next(), split.next()) {
                (Some(one), Some(two), None) => Ok((one, two)),
                _ => Err(StatusCode::UNAUTHORIZED),
            }
        }

        fn insert_payload<I: IdentityProvider, B>(
            req: &mut Request<B>,
            payload: Option<Payload>,
        ) -> Result<(), StatusCode> {
            if let Some(payload) = payload {
                trace!("Write sub, group, extra into request.");
                let extensions = req.extensions_mut();
                extensions.insert(I::Id::from(payload.sub));
                extensions.insert(I::Group::from(payload.group));
                extensions.insert(
                    I::Extra::try_from(payload.extra).map_err(|_| StatusCode::BAD_GATEWAY)?,
                );
            }
            Ok(())
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
                        let key = KEY.get().expect("Cookie key is not initialized");
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

                    // TODO: Make cookie authorization safer?
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

                    insert_payload::<I, _>(&mut req, res.payload)?;

                    return Ok((req, Some(cookie_jar)));
                }

                insert_payload::<I, _>(&mut req, res.payload)?;

                Ok((req, None))
            }

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

                    insert_payload::<I, _>(&mut req, res.payload)?;

                    return Ok((req, Some(header_map)));
                }

                insert_payload::<I, _>(&mut req, res.payload)?;

                Ok((req, None))
            }
        }

        impl Method {
            pub(super) fn auth<I: IdentityProvider, B: Send + 'static, ResBody: Default + Send>(
                self,
                client: TokenServiceClient<Channel>,
                req: Request<B>,
            ) -> BoxFut<B, ResBody> {
                Box::pin(async move {
                    let (res, www) = match self {
                        Method::CookieAuth { auth_conf, www } => (
                            Method::cookie_auth::<I, _>(client, auth_conf, req)
                                .await
                                .map(|(req, cookies)| match cookies {
                                    None => (req, None),
                                    Some(cookie_jar) => {
                                        let mut header_map = HeaderMap::new();
                                        write_cookie(&mut header_map, &cookie_jar);
                                        (req, Some(header_map))
                                    }
                                }),
                            www,
                        ),
                        Method::BearerAuth { header, www } => (
                            Method::bearer_auth::<I, _>(
                                client,
                                header
                                    .as_str()
                                    .parse()
                                    .expect("Not a valid bearer http header"),
                                req,
                            )
                            .await,
                            www,
                        ),
                    };
                    res.map_err(|code| {
                        Response::builder()
                            .status(code)
                            .header(header::WWW_AUTHENTICATE, www.as_str())
                            .body(ResBody::default())
                            .unwrap()
                    })
                })
            }
        }
    }
}

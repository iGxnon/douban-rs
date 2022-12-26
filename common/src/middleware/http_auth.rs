use crate::consts;
use cookie::time::Duration;
use cookie::{Cookie, SameSite};
use futures::future::BoxFuture;
use http::{header, HeaderValue, Request, Response, StatusCode};
use pin_project_lite::pin_project;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tower::{Layer, Service};
use tracing::{event, instrument, span, Instrument, Level};

// U 是验证用户身份的东西，如 Uid
pub trait AuthBackend<U>: Clone {
    // 硬编码(调试) 方式进行认证，默认禁用
    fn auth_basic(&self, _username: &str, _password: &str) -> Result<U, String> {
        Err("invalid authorization".to_string())
    }

    // 数据库/缓存 查询方式进行认证，默认禁用，其他服务自行实现 (最好使用异步 api 实现)
    // 可以更新 Cookie
    fn auth_basic_async(
        &self,
        _username: &str,
        _password: &str,
    ) -> BoxFuture<'static, Result<(U, Option<String>), String>> {
        Box::pin(async { Err("invalid authorization".to_string()) })
    }

    // 使用 auth 服务进行认证，其他服务自行实现与 auth 服务的通讯 (最好使用异步 api 实现)
    // 认证 Bearer 时可能需要更新 token，所以返回 Ok(UserId, Option<HeaderValue>)
    fn auth_bearer(&self, token: &str) -> BoxFuture<'static, Result<(U, Option<String>), String>>;

    // TODO auth digest
}

#[derive(Copy, Clone)]
pub struct HttpAuthLayer<A, U> {
    backend: A,
    disable_cookie: bool,
    enable_basic_auth: bool,
    cookie_max_age: i64,
    _data: PhantomData<U>,
}

impl<A, U> HttpAuthLayer<A, U>
where
    A: AuthBackend<U>,
{
    pub fn new(
        backend: A,
        enable_basic_auth: bool,
        disable_cookie: bool,
        cookie_max_age: i64,
    ) -> HttpAuthLayer<A, U> {
        Self {
            backend,
            disable_cookie,
            enable_basic_auth,
            cookie_max_age,
            _data: PhantomData::default(),
        }
    }

    pub fn without_cookie(backend: A, enable_basic_auth: bool) -> Self {
        Self {
            backend,
            disable_cookie: true,
            enable_basic_auth,
            cookie_max_age: 0,
            _data: PhantomData::default(),
        }
    }
}

impl<A, U> Default for HttpAuthLayer<A, U>
where
    A: Default,
{
    fn default() -> Self {
        Self {
            backend: A::default(),
            disable_cookie: false,
            enable_basic_auth: false,
            cookie_max_age: 2 * 24 * 60 * 60, // 2d
            _data: PhantomData::default(),
        }
    }
}

impl<A, S, U> Layer<S> for HttpAuthLayer<A, U>
where
    A: AuthBackend<U>,
{
    type Service = HttpAuth<A, S, U>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            backend: self.backend.clone(),
            disable_cookie: self.disable_cookie,
            enable_basic_auth: self.enable_basic_auth,
            cookie_max_age: self.cookie_max_age,
            _data: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct HttpAuth<A, S, U> {
    inner: S,
    backend: A,
    disable_cookie: bool,
    enable_basic_auth: bool,
    cookie_max_age: i64,
    _data: PhantomData<U>,
}

#[derive(Debug, Clone)]
struct SetCookie(String);

impl<A, S, ReqBody, ResBody, U> Service<Request<ReqBody>> for HttpAuth<A, S, U>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone,
    ResBody: Default,
    A: AuthBackend<U>,
    U: Display + Send + Sync + 'static,
    ReqBody: Debug + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<
        S,
        BoxFuture<'static, Result<Request<ReqBody>, S::Response>>,
        ReqBody,
        ResBody,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[instrument(level = "trace", skip(self), name = "middleware")]
    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let mut unauthorized_resp = Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(header::WWW_AUTHENTICATE, consts::WWW_AUTH_BEARER);
        if self.enable_basic_auth {
            unauthorized_resp =
                unauthorized_resp.header(header::WWW_AUTHENTICATE, consts::WWW_AUTH_BASIC);
        }
        if !self.disable_cookie {
            unauthorized_resp =
                unauthorized_resp.header(header::WWW_AUTHENTICATE, consts::WWW_AUTH_COOKIE);
        }
        let token = parse_token(&req, self.disable_cookie, self.enable_basic_auth);
        if let Some((scheme, value)) = token {
            let auth = auth_token(scheme, value, &self.backend);
            let cookie_enable = !self.disable_cookie;
            let auth = async move {
                return match auth.await {
                    Ok((u, refresh)) => {
                        event!(
                            Level::TRACE,
                            target = "middleware:auth",
                            "authorized, {}",
                            u
                        );
                        req.extensions_mut().insert(u);
                        if cookie_enable {
                            if let Some(cookie) = refresh {
                                req.extensions_mut().insert(SetCookie(cookie));
                            }
                        }
                        Ok(req)
                    }
                    Err(reason) => {
                        event!(
                            Level::INFO,
                            target = "middleware:auth",
                            "unable to authorize, cause {}",
                            reason
                        );
                        Err(unauthorized_resp.body(ResBody::default()).unwrap())
                    }
                };
            }
            .instrument(span!(Level::INFO, "auth"));
            return ResponseFuture {
                state: State::Authorize {
                    authorize: Box::pin(auth),
                    max_age: self.cookie_max_age,
                },
                service: self.inner.clone(),
            };
        }
        // unauthorized cause no credentials found
        event!(
            Level::INFO,
            target = "middleware:auth",
            "unable to authorize, cause no credentials found"
        );
        ResponseFuture {
            state: State::Authorize {
                authorize: Box::pin(async move {
                    Err(unauthorized_resp.body(ResBody::default()).unwrap())
                }),
                max_age: self.cookie_max_age,
            },
            service: self.inner.clone(),
        }
    }
}

fn parse_token<B>(
    request: &Request<B>,
    disable_cookie: bool,
    enable_basic_auth: bool,
) -> Option<(&str, &str)> {
    let header = request
        .headers()
        .get(consts::AUTH_HEADER)
        .and_then(|it| it.to_str().ok())
        .and_then(|it| {
            let token = it
                .trim_start_matches(consts::AUTH_SCHEME_BEARER)
                .trim_start_matches(consts::AUTH_SCHEME_BASIC)
                .trim();
            if it.starts_with(consts::AUTH_SCHEME_BEARER) {
                Some((consts::AUTH_SCHEME_BEARER, token))
            } else if it.starts_with(consts::AUTH_SCHEME_BASIC) && enable_basic_auth {
                Some((consts::AUTH_SCHEME_BASIC, token))
            } else {
                None
            }
        });
    if disable_cookie {
        return header;
    }
    if header.is_some() {
        return header;
    }
    request
        .headers()
        .get(header::COOKIE)
        .and_then(|it| {
            it.to_str().ok().map(|cookies| {
                cookies.split(';').map(str::trim).filter(|cookie| {
                    cookie.starts_with(consts::AUTH_COOKIE) // 大小写敏感
                })
            })
        })
        .and_then(|cookies| {
            cookies
                .last()
                .and_then(|cookie| {
                    cookie
                        .strip_prefix(consts::AUTH_COOKIE)
                        .unwrap()
                        .trim()
                        .strip_prefix('=')
                        .map(str::trim)
                })
                .map(|cookie| (header::COOKIE.as_str(), cookie))
        })
}

// 使用提供的后端进行认证
fn auth_token<A, U>(
    scheme: &str,
    token: &str,
    backend: &A,
) -> BoxFuture<'static, Result<(U, Option<String>), String>>
where
    A: AuthBackend<U>,
    U: Send + Sync + 'static,
{
    if scheme == consts::AUTH_SCHEME_BEARER || scheme == header::COOKIE.as_str() {
        return backend.auth_bearer(token);
    }
    if scheme == consts::AUTH_SCHEME_BASIC {
        let decode_bytes = base64::decode(token);
        if decode_bytes.is_err() {
            return Box::pin(async {
                Err("authorization failed! cannot parse `Basic` credential".to_string())
            });
        }
        let decode_str = String::from_utf8(decode_bytes.unwrap());
        if decode_str.is_err() {
            return Box::pin(async {
                Err("authorization failed! cannot parse `Basic` credential".to_string())
            });
        }
        let decode_str = decode_str.unwrap();
        let splits: Vec<_> = decode_str.split(':').collect();
        let username = splits.first();
        if username.is_none() {
            return Box::pin(async {
                Err("authorization failed! cannot found `username` in Basic".to_string())
            });
        }
        let password = splits.get(1);
        if password.is_none() {
            return Box::pin(async {
                Err("authorization failed! cannot found `password` in Basic".to_string())
            });
        }
        if let Ok(u) = backend.auth_basic(username.unwrap(), password.unwrap()) {
            return Box::pin(async move { Ok((u, None)) });
        }
        return backend.auth_basic_async(username.unwrap(), password.unwrap());
    }
    Box::pin(async { Err("no scheme found! unauthorized".to_string()) })
}

pin_project! {
    pub struct ResponseFuture<S, AFut, ReqBody, ResBody>
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>>,
        AFut: Future<Output=Result<Request<ReqBody>, S::Response>>
    {
        #[pin]
        state: State<AFut, S::Future>,
        service: S,
    }
}

pin_project! {
    #[project = StateProj]
    enum State<AFut, SFut> {
        Authorize {
            #[pin]
            authorize: AFut,
            max_age: i64
        },
        Authorized {
            #[pin]
            fut: SFut,
            cookie: Option<SetCookie>,
            max_age: i64
        }
    }
}

impl<S, AFut, ReqBody, ResBody> Future for ResponseFuture<S, AFut, ReqBody, ResBody>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    AFut: Future<Output = Result<Request<ReqBody>, S::Response>>,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        loop {
            match this.state.as_mut().project() {
                StateProj::Authorize { authorize, max_age } => {
                    let max_age = *max_age;
                    let auth = ready!(authorize.poll(cx));
                    match auth {
                        Ok(mut req) => {
                            let cookie = req.extensions().get::<SetCookie>().map(ToOwned::to_owned);
                            req.extensions_mut().remove::<SetCookie>();
                            let fut = this.service.call(req);
                            this.state.set(State::Authorized {
                                fut,
                                cookie,
                                max_age,
                            })
                        }
                        Err(res) => return Poll::Ready(Ok(res)),
                    }
                }
                StateProj::Authorized {
                    fut,
                    cookie,
                    max_age,
                } => {
                    if cookie.is_none() {
                        return fut.poll(cx);
                    }
                    let cookie = cookie.as_ref().unwrap().clone().0;
                    let cookie = Cookie::build(consts::AUTH_COOKIE, cookie)
                        .http_only(true)
                        .same_site(SameSite::None)
                        .secure(true)
                        .max_age(Duration::seconds(*max_age))
                        .path("/")
                        .finish()
                        .to_string();
                    let cookie = match HeaderValue::from_str(&cookie) {
                        Ok(value) => value,
                        Err(_) => return fut.poll(cx),
                    };

                    let result = ready!(fut.poll(cx));
                    return match result {
                        Ok(mut res) => {
                            res.headers_mut().insert(header::SET_COOKIE, cookie);
                            Poll::Ready(Ok(res))
                        }
                        Err(err) => Poll::Ready(Err(err)),
                    };
                }
            }
        }
    }
}

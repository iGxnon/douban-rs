use crate::config::env::optional;
use base64::Engine;
use cookie::{Cookie, CookieJar};
use futures::future::BoxFuture;
use futures::ready;
use http::{HeaderMap, Request, Response};
use pin_project_lite::pin_project;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fmt::Formatter;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

const DEFAULT_MAX_AGE: i64 = 2 * 24 * 60 * 60; // 2 days
pub const DEFAULT_COOKIE_NAME: &str = "x-token";
pub const DEFAULT_COOKIE_PATH: &str = "/";
pub const DEFAULT_COOKIE_DOMAIN: &str = "";

// wrap serde traits for cookie::SameSite
#[derive(Clone, Debug)]
pub struct SameSite(cookie::SameSite);

impl From<SameSite> for cookie::SameSite {
    fn from(value: SameSite) -> Self {
        value.0
    }
}

impl SameSite {
    pub fn new(same_site: cookie::SameSite) -> Self {
        Self(same_site)
    }
}

impl Default for SameSite {
    fn default() -> Self {
        Self::new(cookie::SameSite::None)
    }
}

impl Serialize for SameSite {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            cookie::SameSite::Strict => serializer.serialize_str("strict"),
            cookie::SameSite::Lax => serializer.serialize_str("lax"),
            cookie::SameSite::None => serializer.serialize_str("none"),
        }
    }
}

impl<'de> Deserialize<'de> for SameSite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SameSiteVisitor;
        impl<'vi> serde::de::Visitor<'vi> for SameSiteVisitor {
            type Value = SameSite;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "expect a str in [None, Strict, Lax] or empty")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    "None" | "none" => Ok(SameSite(cookie::SameSite::None)),
                    "Strict" | "strict" => Ok(SameSite(cookie::SameSite::Strict)),
                    "Lax" | "lax" => Ok(SameSite(cookie::SameSite::Lax)),
                    _ => Ok(SameSite(cookie::SameSite::None)),
                }
            }
        }
        deserializer.deserialize_str(SameSiteVisitor)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct CookieAuthConf {
    pub max_age: i64,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: SameSite,
    pub path: String,
    pub domain: String,
    pub encrypted: Option<String>,
    pub cookie_name: String,
}

impl Default for CookieAuthConf {
    fn default() -> Self {
        Self {
            max_age: DEFAULT_MAX_AGE,
            http_only: true,
            secure: true,
            same_site: Default::default(),
            path: DEFAULT_COOKIE_PATH.to_string(),
            domain: DEFAULT_COOKIE_DOMAIN.to_string(),
            encrypted: None,
            cookie_name: DEFAULT_COOKIE_NAME.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct HttpAuthLayer<Auth> {
    auth: Auth,
}

impl<Auth> HttpAuthLayer<Auth> {
    pub fn new(auth: Auth) -> Self {
        Self { auth }
    }
}

#[derive(Clone, Debug)]
pub struct HttpAuth<S, Auth> {
    inner: S,
    auth: Auth,
}

impl<S, Auth> Layer<S> for HttpAuthLayer<Auth>
where
    Auth: Clone,
{
    type Service = HttpAuth<S, Auth>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            auth: self.auth.clone(),
        }
    }
}

pub(crate) fn expect_two<I: Iterator>(mut split: I) -> Option<(I::Item, I::Item)> {
    match (split.next(), split.next(), split.next()) {
        (Some(one), Some(two), None) => Some((one, two)),
        _ => None,
    }
}

pub fn scan_cookies<ReqBody>(req: &Request<ReqBody>) -> CookieJar {
    let mut jar = CookieJar::new();
    req.headers().get(http::header::COOKIE).and_then(|it| {
        it.to_str().ok().map(|cookies| {
            cookies
                .split(';')
                .map(str::trim)
                .map(Cookie::parse_encoded)
                .for_each(|cookie| {
                    if let Ok(cookie) = cookie {
                        jar.add_original(cookie.into_owned())
                    }
                })
        })
    });
    jar
}

pub fn scan_bearer<'a, B>(req: &'a Request<B>, auth_header: &str) -> Option<&'a str> {
    req.headers().get(auth_header).and_then(|v| {
        v.to_str()
            .ok()
            .map(|v| v.trim_start_matches("Bearer ").trim())
    })
}

pub fn scan_basic<B>(req: &Request<B>, auth_header: &str) -> Option<(String, String)> {
    req.headers()
        .get(auth_header)
        .and_then(|v| {
            v.to_str()
                .ok()
                .map(|v| v.trim_start_matches("Basic ").trim())
        })
        .and_then(|v| base64::prelude::BASE64_STANDARD.decode(v).ok())
        .and_then(|v| String::from_utf8(v).ok())
        .and_then(|v| {
            expect_two(v.split(':')).map(|(name, pwd)| (name.to_string(), pwd.to_string()))
        })
}

pub fn write_cookie(res_header: &mut HeaderMap, cookie_jar: &CookieJar) {
    cookie_jar.delta().for_each(|cookie| {
        let cookie = cookie.encoded().to_string();
        res_header.append(http::header::SET_COOKIE, cookie.parse().unwrap());
    });
}

pub trait Auth<B> {
    type ResponseBody;

    fn authorize(
        &mut self,
        req: &mut Request<B>,
    ) -> Result<Option<HeaderMap>, Response<Self::ResponseBody>>;
}

#[derive(Clone, Debug)]
pub struct AsyncHttpAuthLayer<Auth> {
    auth: Auth,
}

impl<Auth> AsyncHttpAuthLayer<Auth> {
    pub fn new(auth: Auth) -> Self {
        Self { auth }
    }
}

#[derive(Clone, Debug)]
pub struct AsyncHttpAuth<S, Auth> {
    inner: S,
    auth: Auth,
}

impl<S, Auth> Layer<S> for AsyncHttpAuthLayer<Auth>
where
    Auth: Clone,
{
    type Service = AsyncHttpAuth<S, Auth>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            auth: self.auth.clone(),
        }
    }
}

pub trait AsyncAuth<B> {
    type RequestBody;
    type ResponseBody;
    type Future: Future<
        Output = Result<
            (Request<Self::RequestBody>, Option<HeaderMap>),
            Response<Self::ResponseBody>,
        >,
    >;

    fn authorize(&mut self, req: Request<B>) -> Self::Future;
}

pub(crate) async fn write_fut<Res, E>(
    fut: impl Future<Output = Result<Response<Res>, E>>,
    headers: HeaderMap,
) -> Result<Response<Res>, E> {
    match fut.await {
        Ok(mut res) => {
            let res_headers = res.headers_mut();
            for (name, value) in headers {
                res_headers.insert(name.unwrap(), value);
            }
            Ok(res)
        }
        Err(err) => Err(err),
    }
}

impl<S, ReqBody, ResBody, A> Service<Request<ReqBody>> for HttpAuth<S, A>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    A: Auth<ReqBody, ResponseBody = ResBody>,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // authorize request
        let header = match self.auth.authorize(&mut req) {
            Ok(header) => header,
            Err(res) => return Box::pin(async move { Ok(res) }),
        };

        let fut = self.inner.call(req);
        match header {
            None => Box::pin(fut),
            Some(headers) => Box::pin(write_fut(fut, headers)),
        }
    }
}

impl<S, Auth, ReqBody, ResBody> Service<Request<ReqBody>> for AsyncHttpAuth<S, Auth>
where
    Auth: AsyncAuth<ReqBody, ResponseBody = ResBody>,
    S: Service<Request<Auth::RequestBody>, Response = Response<ResBody>> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S, ReqBody, Auth>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let authorize = self.auth.authorize(req);
        let inner = self.inner.clone();
        ResponseFuture {
            state: State::Authorize { authorize },
            inner,
        }
    }
}

pin_project! {
    pub struct ResponseFuture<S, ReqBody, Auth>
    where
        S: Service<Request<Auth::RequestBody>>,
        Auth: AsyncAuth<ReqBody>,
    {
        #[pin]
        state: State<Auth::Future, S::Future>,
        inner: S,
    }
}

pin_project! {
    #[project = StateProj]
    enum State<AFut, SFut> {
        Authorize {
            #[pin]
            authorize: AFut,
        },
        Authorized {
            #[pin]
            inner: SFut,
            headers: Option<HeaderMap>,
        },
    }
}

impl<S, ReqBody, ResBody, Auth> Future for ResponseFuture<S, ReqBody, Auth>
where
    S: Service<Request<Auth::RequestBody>, Response = Response<ResBody>>,
    Auth: AsyncAuth<ReqBody, ResponseBody = ResBody>,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        loop {
            match this.state.as_mut().project() {
                StateProj::Authorize { authorize } => {
                    let auth = ready!(authorize.poll(cx));
                    match auth {
                        Ok((req, headers)) => {
                            let inner = this.inner.call(req);
                            this.state.set(State::Authorized { inner, headers })
                        }
                        Err(resp) => return Poll::Ready(Ok(resp)),
                    }
                }
                StateProj::Authorized { inner, headers } => {
                    let res: Result<S::Response, S::Error> = ready!(inner.poll(cx));
                    return match res {
                        Ok(mut resp) => match headers {
                            None => Poll::Ready(Ok(resp)),
                            Some(headers) => {
                                let res_headers = resp.headers_mut();
                                for (name, value) in headers {
                                    res_headers.insert(name, value.clone());
                                }
                                Poll::Ready(Ok(resp))
                            }
                        },
                        Err(err) => Poll::Ready(Err(err)),
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::layer::http_auth::{AsyncAuth, AsyncHttpAuthLayer, Auth, HttpAuthLayer};
    use crate::layer::{scan_basic, scan_bearer};
    use futures::future::BoxFuture;
    use http::{header, status::StatusCode, HeaderMap, Request, Response};
    use tower::{BoxError, ServiceBuilder, ServiceExt};

    #[derive(Clone, Debug)]
    struct MyAuth {
        auth_header: String,
    }

    impl<B> Auth<B> for MyAuth {
        type ResponseBody = &'static str;

        fn authorize(
            &mut self,
            req: &mut Request<B>,
        ) -> Result<Option<HeaderMap>, Response<Self::ResponseBody>> {
            if let Some((name, pwd)) = scan_basic(req, self.auth_header.as_str()) {
                if name == "admin" && pwd == "admin" {
                    return Ok(None);
                }
                return Err(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body("unauthorized basic")
                    .unwrap());
            }
            Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("cannot parse basic")
                .unwrap())
        }
    }

    async fn handle(request: Request<&'static str>) -> Result<Response<&'static str>, BoxError> {
        Ok(Response::new(request.body()))
    }

    #[tokio::test]
    async fn test_my_auth() {
        let svc = ServiceBuilder::new()
            .boxed()
            .layer(HttpAuthLayer::new(MyAuth {
                auth_header: header::AUTHORIZATION.to_string(),
            }))
            .service_fn(handle);
        let resp = svc
            .oneshot(
                Request::builder()
                    .header(header::AUTHORIZATION, "Basic YWRtaW46YWRtaW4=")
                    .body("hi")
                    .unwrap(),
            )
            .await;
        println!("{:?}", resp);
    }

    #[derive(Clone, Debug)]
    struct MyAsyncAuth {
        auth_header: String,
    }

    impl<B> AsyncAuth<B> for MyAsyncAuth
    where
        B: Send + 'static,
    {
        type RequestBody = B;
        type ResponseBody = &'static str;
        type Future = BoxFuture<
            'static,
            Result<(Request<Self::RequestBody>, Option<HeaderMap>), Response<Self::ResponseBody>>,
        >;

        fn authorize(&mut self, req: Request<B>) -> Self::Future {
            let bearer = scan_bearer(&req, self.auth_header.as_str()).map(str::to_string);
            Box::pin(async move {
                match bearer {
                    None => Err(Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body("cannot find bearer")
                        .unwrap()),
                    Some(bearer) => {
                        if bearer == "token" {
                            return Ok((req, None));
                        }
                        if bearer == "old-token" {
                            let iter = vec![(
                                "set-authorization".parse().unwrap(),
                                "token".parse().unwrap(),
                            )]
                            .into_iter();
                            return Ok((req, Some(HeaderMap::from_iter(iter))));
                        }
                        Err(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body("cannot find bearer")
                            .unwrap())
                    }
                }
            })
        }
    }

    #[tokio::test]
    async fn test_my_async_auth() {
        let svc = ServiceBuilder::new()
            .boxed()
            .layer(AsyncHttpAuthLayer::new(MyAsyncAuth {
                auth_header: header::AUTHORIZATION.to_string(),
            }))
            .service_fn(handle);
        let resp = svc
            .oneshot(
                Request::builder()
                    .header(header::AUTHORIZATION, "Bearer old-token")
                    .body("hi")
                    .unwrap(),
            )
            .await;
        println!("{:?}", resp);
    }
}

use crate::config::env::optional;
use cookie::{Cookie, CookieJar};
use futures::future::BoxFuture;
use http::{Request, Response};
use pin_project_lite::pin_project;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::future::Future;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tower::{Layer, Service};

const DEFAULT_MAX_AGE: i64 = 2 * 24 * 60 * 60; // 2 days

// wrap serde traits for cookie::SameSite
#[derive(Clone, Debug)]
pub struct SameSite(cookie::SameSite);

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
}

impl Default for CookieAuthConf {
    fn default() -> Self {
        Self {
            max_age: DEFAULT_MAX_AGE,
            http_only: true,
            secure: true,
            same_site: Default::default(),
            path: optional("COOKIE_AUTH_PATH", "/"),
            domain: optional("COOKIE_AUTH_DOMAIN", ""),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CookieAuthLayer<Auth> {
    auth: Auth,
}

impl<Auth> CookieAuthLayer<Auth> {
    pub fn new(auth: Auth) -> Self {
        Self { auth }
    }
}

impl<S, Auth> Layer<S> for CookieAuthLayer<Auth>
where
    Auth: Clone,
{
    type Service = CookieAuth<S, Auth>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            auth: self.auth.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CookieAuth<S, Auth> {
    inner: S,
    auth: Auth,
}

pub(crate) fn scan_cookies<ReqBody>(req: &Request<ReqBody>) -> CookieJar {
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

pub(crate) async fn write_cookie<Res, E>(
    fut: impl Future<Output = Result<Response<Res>, E>>,
    cookie_jar: CookieJar,
) -> Result<Response<Res>, E> {
    match fut.await {
        Ok(mut res) => {
            let headers = res.headers_mut();
            cookie_jar.delta().for_each(|cookie| {
                let cookie = cookie.encoded().to_string();
                headers.append(http::header::SET_COOKIE, cookie.parse().unwrap());
            });
            Ok(res)
        }
        Err(err) => Err(err),
    }
}

pub trait AuthCookie<B> {
    type ResponseBody;

    fn authorize(
        &mut self,
        req: &mut Request<B>,
        cookie_jar: &mut CookieJar,
    ) -> Result<(), Response<Self::ResponseBody>>;
}

impl<B, F, ResBody> AuthCookie<B> for F
where
    F: Fn(&mut Request<B>, &mut CookieJar) -> Result<(), Response<ResBody>>,
{
    type ResponseBody = ResBody;

    fn authorize(
        &mut self,
        req: &mut Request<B>,
        cookie_jar: &mut CookieJar,
    ) -> Result<(), Response<Self::ResponseBody>> {
        self(req, cookie_jar)
    }
}

impl<S, ReqBody, ResBody, Auth> Service<Request<ReqBody>> for CookieAuth<S, Auth>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    Auth: AuthCookie<ReqBody, ResponseBody = ResBody>,
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
        let mut cookie_jar = scan_cookies(&req);

        // authorize cookies
        if let Err(resp) = self.auth.authorize(&mut req, &mut cookie_jar) {
            return Box::pin(async move { Ok(resp) });
        };

        let fut = self.inner.call(req);
        Box::pin(write_cookie(fut, cookie_jar))
    }
}

#[derive(Clone, Debug)]
pub struct AsyncCookieAuthLayer<Auth> {
    auth: Auth,
}

impl<Auth> AsyncCookieAuthLayer<Auth> {
    pub fn new(auth: Auth) -> Self {
        Self { auth }
    }
}

impl<S, Auth> Layer<S> for AsyncCookieAuthLayer<Auth>
where
    Auth: Clone,
{
    type Service = AsyncCookieAuth<S, Auth>;

    fn layer(&self, inner: S) -> Self::Service {
        Self::Service {
            inner,
            auth: self.auth.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AsyncCookieAuth<S, Auth> {
    inner: S,
    auth: Auth,
}

pub trait AsyncAuthCookie<B> {
    type RequestBody;
    type ResponseBody;
    type Future: Future<
        Output = Result<(Request<Self::RequestBody>, CookieJar), Response<Self::ResponseBody>>,
    >;

    fn authorize(&mut self, req: Request<B>, cookie_jar: CookieJar) -> Self::Future;
}

impl<B, F, ReqBody, ResBody, Fut> AsyncAuthCookie<B> for F
where
    F: Fn(Request<B>, CookieJar) -> Fut,
    Fut: Future<Output = Result<(Request<ReqBody>, CookieJar), Response<ResBody>>>,
{
    type RequestBody = ReqBody;
    type ResponseBody = ResBody;
    type Future = Fut;

    fn authorize(&mut self, req: Request<B>, cookie_jar: CookieJar) -> Self::Future {
        self(req, cookie_jar)
    }
}

impl<S, Auth, ReqBody, ResBody> Service<Request<ReqBody>> for AsyncCookieAuth<S, Auth>
where
    Auth: AsyncAuthCookie<ReqBody, ResponseBody = ResBody>,
    S: Service<Request<Auth::RequestBody>, Response = Response<ResBody>> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S, ReqBody, Auth>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let cookie_jar = scan_cookies(&req);
        let authorize = self.auth.authorize(req, cookie_jar);
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
        Auth: AsyncAuthCookie<ReqBody>,
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
            authorize: AFut
        },
        Authorized {
            #[pin]
            inner: SFut,
            cookie_jar: CookieJar,
        }
    }
}

impl<S, ReqBody, ResBody, Auth> Future for ResponseFuture<S, ReqBody, Auth>
where
    S: Service<Request<Auth::RequestBody>, Response = Response<ResBody>>,
    Auth: AsyncAuthCookie<ReqBody, ResponseBody = ResBody>,
{
    type Output = Result<S::Response, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        loop {
            match this.state.as_mut().project() {
                StateProj::Authorize { authorize } => {
                    let auth: Result<(Request<Auth::RequestBody>, CookieJar), S::Response> =
                        ready!(authorize.poll(cx));
                    match auth {
                        Ok((req, cookie_jar)) => {
                            let inner = this.inner.call(req);
                            this.state.set(State::Authorized { inner, cookie_jar })
                        }
                        Err(resp) => return Poll::Ready(Ok(resp)),
                    }
                }
                StateProj::Authorized { inner, cookie_jar } => {
                    let fut: Result<S::Response, S::Error> = ready!(inner.poll(cx));
                    return match fut {
                        Ok(mut resp) => {
                            let headers = resp.headers_mut();
                            cookie_jar.delta().for_each(|cookie| {
                                let cookie = cookie.encoded().to_string();
                                headers.append(http::header::SET_COOKIE, cookie.parse().unwrap());
                            });
                            Poll::Ready(Ok(resp))
                        }
                        Err(err) => Poll::Ready(Err(err)),
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cookie::time::Duration;
    use tower::{BoxError, ServiceBuilder, ServiceExt};

    #[derive(Clone)]
    struct MyCookieAuth(CookieAuthConf);

    impl MyCookieAuth {
        fn new(conf: CookieAuthConf) -> Self {
            Self(conf)
        }
    }

    impl<ReqBody> AuthCookie<ReqBody> for MyCookieAuth {
        type ResponseBody = &'static str;

        fn authorize(
            &mut self,
            req: &mut Request<ReqBody>,
            cookie_jar: &mut CookieJar,
        ) -> Result<(), Response<Self::ResponseBody>> {
            if let Some(token) = cookie_jar.get("x-token") {
                if token.value() == "old token" {
                    cookie_jar.remove(Cookie::named("x-token"));
                    return Ok(());
                }
                return Err(Response::builder()
                    .status(http::status::StatusCode::UNAUTHORIZED)
                    .body("invalid token")
                    .unwrap());
            };
            Err(Response::builder()
                .status(http::status::StatusCode::UNAUTHORIZED)
                .body("no token found")
                .unwrap())
        }
    }

    fn my_cookie_auth_func<B>(
        req: &mut Request<B>,
        cookie_jar: &mut CookieJar,
    ) -> Result<(), Response<&'static str>> {
        if let Some(token) = cookie_jar.get("x-token") {
            if token.value() == "old token" {
                cookie_jar.remove(Cookie::named("x-token"));
                req.extensions_mut().insert("x");
                return Ok(());
            }
            return Err(Response::builder()
                .status(http::status::StatusCode::UNAUTHORIZED)
                .body("invalid token")
                .unwrap());
        };
        Err(Response::builder()
            .status(http::status::StatusCode::UNAUTHORIZED)
            .body("no token found")
            .unwrap())
    }

    #[derive(Clone)]
    struct MyAsyncCookieAuth<F> {
        f: F,
        conf: CookieAuthConf,
    }

    impl<F> MyAsyncCookieAuth<F> {
        fn new(conf: CookieAuthConf, f: F) -> Self {
            Self { f, conf }
        }
    }

    impl<B, F, Fut> AsyncAuthCookie<B> for MyAsyncCookieAuth<F>
    where
        B: Send + 'static,
        F: Fn(CookieAuthConf, Request<B>, CookieJar) -> Fut,
        Fut: Future<Output = Result<(Request<&'static str>, CookieJar), Response<&'static str>>>,
    {
        type RequestBody = &'static str;
        type ResponseBody = &'static str;
        type Future = Fut;

        fn authorize(&mut self, req: Request<B>, cookie_jar: CookieJar) -> Self::Future {
            let func = &self.f;
            func(self.conf.clone(), req, cookie_jar)
        }
    }

    async fn my_async_cookie_auth<B>(
        conf: CookieAuthConf,
        req: Request<B>,
        mut cookie_jar: CookieJar,
    ) -> Result<(Request<&'static str>, CookieJar), Response<&'static str>> {
        if let Some(token) = cookie_jar.get("x-token") {
            if token.value() == "old token" {
                cookie_jar.add(
                    Cookie::build("x-token", "new token")
                        .same_site(conf.same_site.0)
                        .max_age(Duration::seconds(conf.max_age))
                        .path(conf.path)
                        .domain(conf.domain)
                        .finish(),
                );
                let parts = req.into_parts().0;
                return Ok((Request::from_parts(parts, "cookie auth body"), cookie_jar));
            }
            return Err(Response::builder()
                .status(http::status::StatusCode::UNAUTHORIZED)
                .body("invalid token")
                .unwrap());
        };
        Err(Response::builder()
            .status(http::status::StatusCode::UNAUTHORIZED)
            .body("no token found")
            .unwrap())
    }

    async fn my_async_cookie_auth_func<B>(
        req: Request<B>,
        mut cookie_jar: CookieJar,
    ) -> Result<(Request<&'static str>, CookieJar), Response<&'static str>> {
        if let Some(token) = cookie_jar.get("x-token") {
            if token.value() == "old token" {
                cookie_jar.remove(Cookie::named("x-token"));
                let parts = req.into_parts().0;
                return Ok((Request::from_parts(parts, "cookie auth body"), cookie_jar));
            }
            return Err(Response::builder()
                .status(http::status::StatusCode::UNAUTHORIZED)
                .body("invalid token")
                .unwrap());
        };
        Err(Response::builder()
            .status(http::status::StatusCode::UNAUTHORIZED)
            .body("no token found")
            .unwrap())
    }

    #[tokio::test]
    async fn test() {
        let svc = ServiceBuilder::new()
            .boxed()
            .layer(CookieAuthLayer::new(MyCookieAuth::new(
                CookieAuthConf::default(),
            )))
            .service_fn(handle);
        let resp = svc
            .oneshot(
                Request::builder()
                    .header(http::header::COOKIE, "x-token=old token")
                    .body("xx")
                    .unwrap(),
            )
            .await;
        println!("{:?}", resp);
    }

    #[tokio::test]
    async fn test_func() {
        let svc = ServiceBuilder::new()
            .boxed()
            .layer(CookieAuthLayer::new(my_cookie_auth_func))
            .service_fn(handle);
        let resp = svc
            .oneshot(
                Request::builder()
                    .header(http::header::COOKIE, "x-token=old token")
                    .body("xx")
                    .unwrap(),
            )
            .await;
        println!("{:?}", resp);
    }

    #[tokio::test]
    async fn async_test() {
        let svc = ServiceBuilder::new()
            .boxed()
            .layer(AsyncCookieAuthLayer::new(MyAsyncCookieAuth::new(
                CookieAuthConf::default(),
                my_async_cookie_auth,
            )))
            .service_fn(handle);
        let resp = svc
            .oneshot(
                Request::builder()
                    .header(http::header::COOKIE, "x-token=old token")
                    .body("xx")
                    .unwrap(),
            )
            .await;
        println!("{:?}", resp);
    }

    #[tokio::test]
    async fn async_func_test() {
        let svc = ServiceBuilder::new()
            .layer(AsyncCookieAuthLayer::new(my_async_cookie_auth_func))
            .service_fn(handle);
        let resp = svc
            .oneshot(
                Request::builder()
                    .header(http::header::COOKIE, "x-token=old token")
                    .body(1)
                    .unwrap(),
            )
            .await;
        println!("{:?}", resp);
    }

    async fn handle(request: Request<&'static str>) -> Result<Response<&'static str>, BoxError> {
        Ok(Response::new(request.body()))
    }
}

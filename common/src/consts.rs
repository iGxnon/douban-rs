// 定义了一些常量
// TODO delete

use http::header::{self, HeaderName};

// app 名称
pub const APP_NAME: &str = "douban-web";

// HTTP 用于全局认证的请求头，默认为 http::header::AUTHORIZATION
// 需要混淆/自定义返回头可以自行修改
// 认证字段和内容格式：<Authorization>: <auth-scheme> <authorization-parameters>
// 例如：Authorization: Basic <credentials>
// 参考：https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization
pub const AUTH_HEADER: HeaderName = header::AUTHORIZATION;
pub const AUTH_COOKIE: &str = "x-token";
// 参考：https://datatracker.ietf.org/doc/html/rfc6750
// encrypted jwt token
pub const AUTH_SCHEME_BEARER: &str = "Bearer";
// 参考：https://datatracker.ietf.org/doc/html/rfc7617
// base64(<username>:<password>)
pub const AUTH_SCHEME_BASIC: &str = "Basic";

// 三种鉴权方式，当鉴权失败时作为返回中的 WWW-Authenticate header
pub const WWW_AUTH_BASIC: &str = r#"Basic realm=douban-web-http-auth,charset=UTF-8"#;
pub const WWW_AUTH_BEARER: &str = r#"Bearer realm=douban-web-http-auth,charset=UTF-8"#;
pub const WWW_AUTH_COOKIE: &str =
    r#"Cookie realm=douban-web-http-auth,charset=UTF-8,cookie-name=x-token"#;

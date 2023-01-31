use crate::rest::RestResolver;
use axum::http::StatusCode;
use axum::{BoxError, Json};
use common::infra::Resolver;
use common::status::prelude::*;

// handle layer error
pub(in crate::rest) async fn handle_error(err: BoxError) -> (StatusCode, Json<Resp<'static, ()>>) {
    // Timeout
    if err.is::<tower::timeout::error::Elapsed>() {
        return (
            StatusCode::REQUEST_TIMEOUT,
            Json(Resp::failed_detail(ErrorDetail::ErrorInfo {
                reason: "Request timeout",
                domain: RestResolver::DOMAIN,
                metadata: Default::default(),
            })),
        );
    }
    if err.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(Resp::failed_detail(ErrorDetail::ErrorInfo {
                reason: "Load shed because too many request",
                domain: RestResolver::DOMAIN,
                metadata: Default::default(),
            })),
        );
    }
    // Other error
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Resp::failed(HttpStatus::default())),
    )
}

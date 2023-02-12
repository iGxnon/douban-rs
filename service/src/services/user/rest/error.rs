use crate::user::rest::RestResolver;
use axum::http::StatusCode;
use axum::{BoxError, Json};
use common::infra::Resolver;
use common::status::prelude::*;

// handle layer error
pub(super) async fn handle_error(err: BoxError) -> (StatusCode, Json<Resp<()>>) {
    // Timeout
    if err.is::<tower::timeout::error::Elapsed>() {
        return (
            StatusCode::REQUEST_TIMEOUT,
            Json(Resp::failed_detail(
                StatusCode::REQUEST_TIMEOUT,
                ErrorDetail::ErrorInfo {
                    reason: "Request timeout".into(),
                    domain: RestResolver::DOMAIN.into(),
                    metadata: None,
                },
            )),
        );
    }
    // Over loaded
    if err.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(Resp::failed_detail(
                StatusCode::TOO_MANY_REQUESTS,
                ErrorDetail::ErrorInfo {
                    reason: "Load shed because too many request".into(),
                    domain: RestResolver::DOMAIN.into(),
                    metadata: None,
                },
            )),
        );
    }
    // Other error
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Resp::failed_code(StatusCode::INTERNAL_SERVER_ERROR)),
    )
}

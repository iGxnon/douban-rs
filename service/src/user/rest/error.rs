use axum::http::StatusCode;
use axum::{BoxError, Json};
use common::status::prelude::*;

// handle layer error
pub(crate) async fn handle_error(err: BoxError) -> (StatusCode, Json<Resp<()>>) {
    // Timeout
    if err.is::<tower::timeout::error::Elapsed>() {
        return (
            StatusCode::REQUEST_TIMEOUT,
            Json(Resp::failed_message(
                StatusCode::REQUEST_TIMEOUT,
                "Request timeout",
            )),
        );
    }
    // Over loaded
    if err.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(Resp::failed_message(
                StatusCode::TOO_MANY_REQUESTS,
                "Load shed because too many request",
            )),
        );
    }
    // Other error
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Resp::failed_code(StatusCode::INTERNAL_SERVER_ERROR)),
    )
}

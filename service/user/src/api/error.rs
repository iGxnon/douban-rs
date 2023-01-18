use crate::api::Resp;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{BoxError, Json};
use thiserror::Error;
use tracing::error;

pub(in crate::api) type Result<D> = std::result::Result<Json<Resp<D>>, Error>;

#[derive(Debug, Error)]
pub enum Error {
    //////////////////
    // client error //
    //////////////////
    #[error("Wrong password")]
    WrongPassword,

    ////////////////////
    // internal error //
    ////////////////////
    #[error("Rpc error")]
    RpcError,
}

impl IntoResponse for Error {
    // mapping app error
    fn into_response(self) -> Response {
        match self {
            Error::RpcError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Resp::<()>::failed("Unhandled internal error")),
            ),
            _ => (
                StatusCode::BAD_REQUEST,
                Json(Resp::failed(self.to_string())),
            ),
        }
        .into_response()
    }
}

// handle layer(layer) error
pub(in crate::api) async fn handle_error(err: BoxError) -> (StatusCode, Json<Resp<()>>) {
    // Timeout
    if err.is::<tower::timeout::error::Elapsed>() {
        return (
            StatusCode::REQUEST_TIMEOUT,
            Json(Resp::failed("Request took too long")),
        );
    }
    // Other error
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Resp::failed("Unhandled internal error")),
    )
}

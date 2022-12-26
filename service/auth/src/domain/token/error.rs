use thiserror::Error;
use tonic::{Response, Status};

#[derive(Debug, Error)]
pub enum Error {
    #[error("missing payload while generating/parsing new token")]
    NoPayload,
    #[error("encrypt token failed, please check clients secrets config")]
    EncryptFailed,
    #[error("serializer error {0:?}")]
    SerializerError(#[from] serde_json::Error),
    #[error("invalid argument `sid`, no sid found in clients")]
    NoSID,
    #[error("invalid argument `secret`, reason: {0:?}")]
    InvalidSecret(String),
    #[error("invalid argument `id`, expect a proper jsonwebtoken id, reason: {0:?}")]
    InvalidKid(String),
    #[error("invalid token {0:?}")]
    InvalidToken(String),
    #[error("jwt error {0:?}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("redis error {0:?}")]
    RedisError(#[from] redis::RedisError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        match err {
            Error::NoPayload => Status::internal(err.to_string()),
            Error::EncryptFailed => Status::internal(err.to_string()),
            Error::SerializerError(_) => Status::internal(err.to_string()),
            Error::NoSID => Status::invalid_argument(err.to_string()),
            Error::InvalidSecret(_) => Status::invalid_argument(err.to_string()),
            Error::InvalidKid(_) => Status::invalid_argument(err.to_string()),
            Error::InvalidToken(_) => Status::invalid_argument(err.to_string()),
            Error::JwtError(_) => Status::internal(err.to_string()),
            Error::RedisError(_) => Status::internal(err.to_string()),
            Error::Other(_) => Status::unknown(err.to_string()),
        }
    }
}

pub trait ErrorExt<T> {
    fn into_response(self) -> Result<Response<T>, Status>;
}

impl<T> ErrorExt<T> for Result<T, Error> {
    fn into_response(self) -> Result<Response<T>, Status> {
        self.map_or_else(|err| Err(Status::from(err)), |v| Ok(Response::new(v)))
    }
}

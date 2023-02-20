pub(crate) mod bind;
pub(crate) mod login;
pub(crate) mod register;

use crate::user::rest::types::*;
use crate::user::rest::RestResolver;
use axum::extract::State;
use axum::*;
use common::status::prelude::*;
use http::StatusCode;
use proto::pb::user::sys::v1 as pb;
use std::sync::Arc;

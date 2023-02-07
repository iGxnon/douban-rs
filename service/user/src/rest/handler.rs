pub(in crate::rest) mod bind;
pub(in crate::rest) mod login;
pub(in crate::rest) mod register;

use crate::rest::types::*;
use crate::rest::RestResolver;
use axum::extract::State;
use axum::*;
use common::status::prelude::*;
use proto::pb::user::sys::v1 as pb;
use std::sync::Arc;

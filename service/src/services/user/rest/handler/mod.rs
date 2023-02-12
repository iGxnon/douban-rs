pub(in super::super) mod bind;
pub(in super::super) mod login;
pub(in super::super) mod register;

use crate::user::rest::types::*;
use crate::user::rest::RestResolver;
use axum::extract::State;
use axum::*;
use common::status::prelude::*;
use proto::pb::user::sys::v1 as pb;
use std::sync::Arc;

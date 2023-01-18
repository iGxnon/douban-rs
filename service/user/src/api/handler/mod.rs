pub(in crate::api) mod bind;
pub(in crate::api) mod login;
pub(in crate::api) mod register;

use crate::api::error::Error;
use crate::api::types::*;
use crate::api::{error, Resolver};
use axum::extract::State;
use axum::Form;
use std::sync::Arc;
use std::time::Duration;

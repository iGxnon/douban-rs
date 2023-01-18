/// Extent features for tonic status detail.
/// From https://github.com/googleapis/googleapis/blob/master/google/rpc/error_details.proto
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tonic::Status;

pub trait StatusExt {
    fn add_detail(self, detail: ErrorDetail) -> Self;
    fn err_details(&self) -> serde_json::Result<Vec<ErrorDetail>>;
}

fn check_list(details: &[u8]) -> Vec<ErrorDetail> {
    let slice = serde_json::from_slice::<Vec<ErrorDetail>>(details);
    match slice {
        Ok(v) => v,
        _ => Vec::new(),
    }
}

impl StatusExt for Status {
    fn add_detail(self, detail: ErrorDetail) -> Self {
        let mut details = check_list(self.details());
        details.push(detail);
        let details = serde_json::to_vec(&details)
            .expect("unexpect serde_json error, serialize failed on specified type ErrorDetail");
        Status::with_details_and_metadata(
            self.code(),
            self.message(),
            Bytes::from(details),
            self.metadata().clone(),
        )
    }

    fn err_details(&self) -> serde_json::Result<Vec<ErrorDetail>> {
        serde_json::from_slice(self.details())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum ErrorDetail<'a> {
    ErrorInfo {
        reason: &'a str,
        domain: &'a str,
        metadata: HashMap<&'a str, &'a str>,
    },
    RetryInfo {
        retry_delay: Duration,
    },
    DebugInfo {
        stack_entries: Vec<&'a str>,
        detail: &'a str,
    },
    QuotaFailure {
        violations: Vec<QuotaViolation<'a>>,
    },
    PreconditionFailure {
        violations: Vec<PreconditionViolation<'a>>,
    },
    BadRequest {
        field_violations: Vec<FieldViolation<'a>>,
    },
    RequestInfo {
        request_id: &'a str,
        serving_data: &'a str,
    },
    ResourceInfo {
        resource_type: &'a str,
        resource_name: &'a str,
        owner: &'a str,
        description: &'a str,
    },
    Help {
        links: Vec<Link<'a>>,
    },
    LocalizedMessage {
        locale: &'a str,
        message: &'a str,
    },
}

macro_rules! is_implement {
    ( $(
        ( $lower:ident, $upper:ident );
    )+ ) => {
        $(
            pub fn $lower(&self) -> bool {
                matches!(self, Self::$upper { .. })
            }
        )*
    };
}

impl<'a> ErrorDetail<'a> {
    is_implement!(
        (is_error_info, ErrorInfo);
        (is_retry_info, RetryInfo);
        (is_debug_info, DebugInfo);
        (is_quota_failure, QuotaFailure);
        (is_precondition_failure, PreconditionFailure);
        (is_bad_request, BadRequest);
        (is_request_info, RequestInfo);
        (is_resource_info, ResourceInfo);
        (is_help, Help);
        (is_localized_message, LocalizedMessage);
    );
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaViolation<'a> {
    pub subject: &'a str,
    pub description: &'a str,
}

impl<'a> From<(&'a str, &'a str)> for QuotaViolation<'a> {
    fn from(value: (&'a str, &'a str)) -> Self {
        Self {
            subject: value.0,
            description: value.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreconditionViolation<'a> {
    pub r#type: &'a str,
    pub subject: &'a str,
    pub description: &'a str,
}

impl<'a> From<(&'a str, &'a str, &'a str)> for PreconditionViolation<'a> {
    fn from(value: (&'a str, &'a str, &'a str)) -> Self {
        Self {
            r#type: value.0,
            subject: value.1,
            description: value.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldViolation<'a> {
    pub filed: &'a str,
    pub description: &'a str,
}

impl<'a> From<(&'a str, &'a str)> for FieldViolation<'a> {
    fn from(value: (&'a str, &'a str)) -> Self {
        Self {
            filed: value.0,
            description: value.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link<'a> {
    pub description: &'a str,
    pub url: &'a str,
}

pub mod prompt {
    /// ```rust
    /// use common::invalid_argument;
    /// use tracing::event;
    ///
    /// invalid_argument!("aud", "bad_aud", "one of {:?}", vec!["good_aud", "better_aud"]);
    /// invalid_argument!("aud", "bad_aud");
    /// invalid_argument!("aud");
    /// ```
    #[macro_export]
    macro_rules! invalid_argument {
        ($field:expr, $got:expr, $($expect:tt)+) => {
            ::tonic::Status::invalid_argument(format!(
                r#"Request field '{}' is '{}', expected {}."#,
                $field,
                $got,
                format!($($expect)+),
            ))
        };
        ($field:expr, $expect:expr) => {
            ::tonic::Status::invalid_argument(format!(
                r#"Request field '{}' is invalid, expected {}."#,
                $field,
                $expect,
            ))
        };
        ($field:expr) => {
            ::tonic::Status::invalid_argument(format!(
                r#"Request field '{}' is invalid."#,
                $field,
            ))
        };
    }

    /// **PLEASE TAKE CARE OF SENSITIVE MESSAGE FROM BEING EXPOSED**
    ///
    /// ```rust
    /// use common::failed_precondition;
    /// use common::status::*;
    ///
    /// failed_precondition!("Directory {} is not empty, so it cannot be deleted.", "hah");
    /// failed_precondition!([("TOS", "auth.service", "Term of auth service is not accept.").into()].into());
    /// ```
    #[macro_export]
    macro_rules! failed_precondition {
        ($vio:expr) => {
            ::tonic::Status::failed_precondition("Operation failed.").add_detail(
                ErrorDetail::PreconditionFailure {
                    violations: $vio,
                }
            )
        };
        ($($arg:tt)+) => {
            ::tonic::Status::failed_precondition(format!($($arg)+))
        };
    }

    /// ```rust
    /// use common::out_of_range;
    ///
    /// out_of_range!("page", 15);
    /// out_of_range!("page", 2, 15);
    /// ```
    #[macro_export]
    macro_rules! out_of_range {
        ($field:expr, $start:expr, $end:expr) => {
            ::tonic::Status::out_of_range(format!(
                "Parameter '{}' is out of range [{}, {}].",
                $field, $start, $end,
            ))
        };
        ($field:expr, $max:expr) => {
            ::tonic::Status::out_of_range(format!(
                "Parameter '{}' is out of range [0, {}].",
                $field, $max,
            ))
        };
    }

    /// ```rust
    /// use common::status::*;
    /// use common::unauthenticated;
    ///
    /// unauthenticated!("json web token is expired", "bad.apple", [("good", "boy")].into());
    /// unauthenticated!("json web token is expired", "bad.apple");
    /// unauthenticated!();
    /// ```
    #[macro_export]
    macro_rules! unauthenticated {
        ($reason:expr, $domain:expr, $metadata:expr) => {
            ::tonic::Status::unauthenticated("Invalid authentication credentials.").add_detail(
                ErrorDetail::ErrorInfo {
                    reason: $reason,
                    domain: $domain,
                    metadata: $metadata,
                },
            )
        };
        ($reason:expr, $domain:expr) => {
            ::tonic::Status::unauthenticated("Invalid authentication credentials.").add_detail(
                ErrorDetail::ErrorInfo {
                    reason: $reason,
                    domain: $domain,
                    metadata: Default::default(),
                },
            )
        };
        () => {
            ::tonic::Status::unauthenticated("Invalid authentication credentials.")
        };
    }

    /// ```rust
    /// use common::permission_denied;
    ///
    /// permission_denied!("write");
    /// permission_denied!("write", "movie");
    /// ```
    #[macro_export]
    macro_rules! permission_denied {
        ($perm:expr, $src:expr) => {
            ::tonic::Status::permission_denied(format!(
                "Permission '{}' denied on resource '{}'.",
                $perm, $src,
            ))
        };
        ($perm:expr) => {
            ::tonic::Status::permission_denied(format!("Permission '{}' denied.", $perm))
        };
    }

    /// ```rust
    /// use common::not_found;
    ///
    /// not_found!();
    /// not_found!("movie");
    /// ```
    #[macro_export]
    macro_rules! not_found {
        ($src:expr) => {
            ::tonic::Status::not_found(format!("Resource '{}' not found.", $src))
        };
        () => {
            ::tonic::Status::not_found("Resource not found.")
        };
    }

    /// ```rust
    /// use common::status::*;
    /// use common::aborted;
    ///
    /// aborted!("json web token is expired", "bad.apple", [("good", "boy")].into());
    /// aborted!("json web token is expired", "bad.apple");
    /// aborted!();
    /// ```
    #[macro_export]
    macro_rules! aborted {
        ($reason:expr, $domain:expr, $metadata:expr) => {
            ::tonic::Status::aborted("Request aborted.").add_detail(ErrorDetail::ErrorInfo {
                reason: $reason,
                domain: $domain,
                metadata: $metadata,
            })
        };
        ($reason:expr, $domain:expr) => {
            ::tonic::Status::aborted("Request aborted.").add_detail(ErrorDetail::ErrorInfo {
                reason: $reason,
                domain: $domain,
                metadata: Default::default(),
            })
        };
        () => {
            ::tonic::Status::aborted("Request aborted.")
        };
    }

    /// ```rust
    /// use common::already_exists;
    ///
    /// already_exists!();
    /// already_exists!("movie");
    /// ```
    #[macro_export]
    macro_rules! already_exists {
        ($src:expr) => {
            ::tonic::Status::already_exists(format!("Resource '{}' already exists.", $src))
        };
        () => {
            ::tonic::Status::already_exists("Resource already exists.")
        };
    }

    /// ```rust
    /// use common::resource_exhausted;
    /// use common::status::*;
    ///
    /// resource_exhausted!([("auth", "concurrency limit on auth exceed").into()].into());
    /// resource_exhausted!();
    /// ```
    #[macro_export]
    macro_rules! resource_exhausted {
        ($vio:expr) => {
            ::tonic::Status::resource_exhausted("Too many requests.")
                .add_detail(ErrorDetail::QuotaFailure { violations: $vio })
        };
        () => {
            ::tonic::Status::resource_exhausted("Too many requests.")
        };
    }

    /// ```rust
    /// use common::cancelled;
    ///
    /// cancelled!();
    /// ```
    #[macro_export]
    macro_rules! cancelled {
        () => {
            ::tonic::Status::cancelled("Request cancelled by the client.")
        };
    }

    #[macro_export]
    macro_rules! debug_expand {
        (capture, $macr:ident, $mess:literal) => {{
            let bt = format!("{}", std::backtrace::Backtrace::capture());
            let stack_entries: Vec<&str> = bt
                .split('\n')
                .step_by(2)
                .skip(3)
                .map(str::trim)
                .map(|bt| {
                    str::trim_start_matches(bt, |ch: char| {
                        ch.is_numeric() || ch == ':' || ch == ' '
                    })
                })
                .collect();

            ::tonic::Status::$macr($mess).add_detail(ErrorDetail::DebugInfo {
                stack_entries,
                detail: "",
            })
        }};
        (capture, $detail:expr, $macr:ident, $mess:literal) => {{
            let bt = format!("{}", std::backtrace::Backtrace::capture());
            let stack_entries: Vec<&str> = bt
                .split('\n')
                .step_by(2)
                .skip(3)
                .map(str::trim)
                .map(|bt| {
                    str::trim_start_matches(bt, |ch: char| {
                        ch.is_numeric() || ch == ':' || ch == ' '
                    })
                })
                .collect();

            ::tonic::Status::$macr($mess).add_detail(ErrorDetail::DebugInfo {
                stack_entries,
                detail: $detail,
            })
        }};
        ($detail:expr, $stacks:expr, $macr:ident, $mess:literal) => {
            ::tonic::Status::$macr($mess).add_detail(ErrorDetail::DebugInfo {
                stack_entries: $stacks,
                detail: $detail,
            })
        };
        ($detail:expr, $macr:ident, $mess:literal) => {
            ::tonic::Status::$macr($mess).add_detail(ErrorDetail::DebugInfo {
                stack_entries: Default::default(),
                detail: $detail,
            })
        };
        ($macr:ident, $mess:literal) => {
            ::tonic::Status::$macr($mess)
        };
    }

    /// ```rust
    /// use common::*;
    /// use common::status::*;
    /// use std::backtrace::Backtrace;
    ///
    /// let bt = Backtrace::capture();
    /// let bt_str = format!("{}", bt);
    /// let stacks: Vec<&str> = bt_str.split("\n").map(str::trim).collect();
    ///
    /// data_loss!("missing encode key!", stacks);
    /// data_loss!(capture);
    /// data_loss!(capture, "missing encode key!");
    /// data_loss!();
    /// ```
    #[macro_export]
    macro_rules! data_loss {
        (capture) => {
            debug_expand!(capture, data_loss, "Internal error.")
        };
        (capture, $detail:expr) => {
            debug_expand!(capture, $detail, data_loss, "Internal error.")
        };
        ($detail:expr, $stacks:expr) => {
            debug_expand!($detail, $stacks, data_loss, "Internal error.")
        };
        ($detail:expr) => {
            debug_expand!($detail, data_loss, "Internal error.")
        };
        () => {
            debug_expand!(data_loss, "Internal error.")
        };
    }

    /// ```rust
    /// use common::*;
    /// use common::status::*;
    /// use std::backtrace::Backtrace;
    ///
    /// let bt = Backtrace::capture();
    /// let bt_str = format!("{}", bt);
    /// let stacks: Vec<&str> = bt_str.split("\n").map(str::trim).collect();
    ///
    /// unknown!("unknown", stacks);
    /// unknown!(capture);
    /// unknown!(capture, "unknown");
    /// unknown!();
    /// ```
    #[macro_export]
    macro_rules! unknown {
        (capture) => {
            debug_expand!(capture, unknown, "Unknown error.")
        };
        (capture, $detail:expr) => {
            debug_expand!(capture, $detail, unknown, "Unknown error.")
        };
        ($detail:expr, $stacks:expr) => {
            debug_expand!($detail, $stacks, unknown, "Unknown error.")
        };
        ($detail:expr) => {
            debug_expand!($detail, unknown, "Unknown error.")
        };
        () => {
            debug_expand!(unknown, "Unknown error.")
        };
    }

    /// ```rust
    /// use common::*;
    /// use common::status::*;
    /// use std::backtrace::Backtrace;
    ///
    /// let bt = Backtrace::capture();
    /// let bt_str = format!("{}", bt);
    /// let stacks: Vec<&str> = bt_str.split("\n").map(str::trim).collect();
    ///
    /// internal!("internal", stacks);
    /// internal!(capture);
    /// internal!(capture, "internal");
    /// internal!();
    /// ```
    #[macro_export]
    macro_rules! internal {
        (capture) => {
            debug_expand!(capture, internal, "Internal error.")
        };
        (capture, $detail:expr) => {
            debug_expand!(capture, $detail, internal, "Internal error.")
        };
        ($detail:expr, $stacks:expr) => {
            debug_expand!($detail, $stacks, internal, "Internal error.")
        };
        ($detail:expr) => {
            debug_expand!($detail, internal, "Internal error.")
        };
        () => {
            debug_expand!(internal, "Internal error.")
        };
    }

    /// ```rust
    /// use common::not_implemented;
    ///
    /// not_implemented!("GET");
    /// not_implemented!();
    /// ```
    #[macro_export]
    macro_rules! not_implemented {
        ($m:expr) => {
            ::tonic::Status::unimplemented(format!("Method '{}' not implemented.", $m))
        };
        () => {
            ::tonic::Status::unimplemented("Not implemented.")
        };
    }

    /// ```rust
    /// use common::*;
    /// use common::status::*;
    /// use std::backtrace::Backtrace;
    ///
    /// let bt = Backtrace::capture();
    /// let bt_str = format!("{}", bt);
    /// let stacks: Vec<&str> = bt_str.split("\n").map(str::trim).collect();
    ///
    /// unavailable!("unavailable", stacks);
    /// unavailable!(capture);
    /// unavailable!(capture, "unavailable");
    /// unavailable!();
    /// ```
    #[macro_export]
    macro_rules! unavailable {
        (capture) => {
            debug_expand!(capture, unavailable, "Service Unavailable.")
        };
        (capture, $detail:expr) => {
            debug_expand!(capture, $detail, unavailable, "Service Unavailable.")
        };
        ($detail:expr, $stacks:expr) => {
            debug_expand!($detail, $stacks, unavailable, "Service Unavailable.")
        };
        ($detail:expr) => {
            debug_expand!($detail, unavailable, "Service Unavailable.")
        };
        () => {
            debug_expand!(unavailable, "Service Unavailable.")
        };
    }

    /// ```rust
    /// use common::*;
    /// use common::status::*;
    /// use std::backtrace::Backtrace;
    ///
    /// let bt = Backtrace::capture();
    /// let bt_str = format!("{}", bt);
    /// let stacks: Vec<&str> = bt_str.split("\n").map(str::trim).collect();
    ///
    /// deadline_exceeded!("deadline_exceeded", stacks);
    /// deadline_exceeded!(capture);
    /// deadline_exceeded!(capture, "deadline_exceeded");
    /// deadline_exceeded!();
    /// ```
    #[macro_export]
    macro_rules! deadline_exceeded {
        (capture) => {
            debug_expand!(capture, deadline_exceeded, "Gateway timeout.")
        };
        (capture, $detail:expr) => {
            debug_expand!(capture, $detail, deadline_exceeded, "Gateway timeout.")
        };
        ($detail:expr, $stacks:expr) => {
            debug_expand!($detail, $stacks, deadline_exceeded, "Gateway timeout.")
        };
        ($detail:expr) => {
            debug_expand!($detail, deadline_exceeded, "Gateway timeout.")
        };
        () => {
            debug_expand!(deadline_exceeded, "Gateway timeout.")
        };
    }
}

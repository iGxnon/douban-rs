/// Extent features for tonic status detail.
/// From https://github.com/googleapis/googleapis/blob/master/google/rpc/error_details.proto
use bytes::Bytes;
use detail::*;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use tonic::{Code, Status as TonicStatus};

pub mod prelude {
    pub use crate::debug_expand;
    pub use crate::status::detail::*;
    pub use crate::status::ext::*;
}

pub mod detail {
    use super::*;

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
}

pub mod prompt {
    /// ```rust
    /// use common::invalid_argument;
    /// use common::status::prelude::*;
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
    /// use common::status::prelude::*;
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
    /// use common::status::prelude::*;
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
    /// use common::unauthenticated;
    /// use common::status::prelude::*;
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
    /// use common::status::prelude::*;
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
    /// use common::status::prelude::*;
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
    /// use common::aborted;
    /// use common::status::prelude::*;
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
    /// use common::status::prelude::*;
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
    /// use common::status::prelude::*;
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
    /// use common::status::prelude::*;
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
    /// use common::data_loss;
    /// use common::status::prelude::*;
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
    /// use common::unknown;
    /// use common::status::prelude::*;
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
    /// use common::internal;
    /// use common::status::prelude::*;
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
    /// use common::status::prelude::*;
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
    /// use common::unavailable;
    /// use common::status::prelude::*;
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
    /// use common::deadline_exceeded;
    /// use common::status::prelude::*;
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

pub mod ext {
    use super::*;
    use std::any::TypeId;
    use std::sync::Arc;
    use thiserror::Error;

    pub trait StatusExt<'a> {
        fn add_detail(self, detail: ErrorDetail<'a>) -> Self;
    }

    pub trait CodeExt {
        fn to_http_code(&self) -> StatusCode;
    }

    fn check_list(details: &[u8]) -> Vec<ErrorDetail> {
        let slice = serde_json::from_slice::<Vec<ErrorDetail>>(details);
        match slice {
            Ok(v) => v,
            _ => Vec::new(),
        }
    }

    impl<'a> StatusExt<'a> for TonicStatus {
        fn add_detail(self, detail: ErrorDetail) -> Self {
            let mut details = check_list(self.details());
            details.push(detail);
            let details = serde_json::to_vec(&details).expect(
                "unexpect serde_json error, serialize failed on specified type ErrorDetail",
            );
            TonicStatus::with_details_and_metadata(
                self.code(),
                self.message(),
                Bytes::from(details),
                self.metadata().clone(),
            )
        }
    }

    impl CodeExt for Code {
        fn to_http_code(&self) -> StatusCode {
            match self {
                Code::Ok => StatusCode::OK,
                Code::Cancelled => StatusCode::from_u16(499).unwrap(),
                Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
                Code::InvalidArgument => StatusCode::BAD_REQUEST,
                Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
                Code::NotFound => StatusCode::NOT_FOUND,
                Code::AlreadyExists => StatusCode::CONFLICT,
                Code::PermissionDenied => StatusCode::FORBIDDEN,
                Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
                Code::FailedPrecondition => StatusCode::BAD_REQUEST,
                Code::Aborted => StatusCode::CONFLICT,
                Code::OutOfRange => StatusCode::BAD_REQUEST,
                Code::Unimplemented => StatusCode::NOT_IMPLEMENTED,
                Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
                Code::DataLoss => StatusCode::INTERNAL_SERVER_ERROR,
                Code::Unauthenticated => StatusCode::UNAUTHORIZED,
            }
        }
    }

    #[derive(Serialize, Debug, Error, Default)]
    pub struct HttpStatus<E> {
        details: Option<E>,
        #[serde(skip)]
        #[source]
        source: Option<Arc<dyn Error + Send + Sync + 'static>>,
    }

    impl<'a> StatusExt<'a> for HttpStatus<Vec<ErrorDetail<'a>>> {
        fn add_detail(mut self, detail: ErrorDetail<'a>) -> Self {
            let mut errs = self.details.unwrap_or_default();
            errs.push(detail);
            self.details = Some(errs);
            self
        }
    }

    pub type HttpResult<'a, T> = Result<Option<T>, Option<HttpStatus<Vec<ErrorDetail<'a>>>>>;

    impl<'a, T> From<HttpResult<'a, T>> for Resp<'a, T> {
        fn from(value: HttpResult<'a, T>) -> Self {
            match value {
                Ok(data) => Resp {
                    ok: true,
                    data,
                    err: None,
                },
                Err(err) => Resp {
                    ok: false,
                    data: None,
                    err,
                },
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub struct Resp<'a, T> {
        ok: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<T>,
        #[serde(skip_serializing_if = "Option::is_none")]
        err: Option<HttpStatus<Vec<ErrorDetail<'a>>>>,
    }

    impl<'a, T> Resp<'a, T> {
        pub fn ok(data: T) -> Self {
            Self {
                ok: true,
                data: Some(data),
                err: None,
            }
        }

        pub fn failed(status: HttpStatus<Vec<ErrorDetail<'a>>>) -> Self {
            Self {
                ok: false,
                data: None,
                err: Some(status),
            }
        }

        pub fn failed_detail(detail: ErrorDetail<'a>) -> Self {
            Self::failed(HttpStatus::default().add_detail(detail))
        }
    }

    pub type GrpcResult<T> = Result<T, GrpcStatus>;

    /// A wrapper for [tonic::Status] to get a better debug info with error details
    pub struct GrpcStatus(TonicStatus);

    impl Deref for GrpcStatus {
        type Target = TonicStatus;

        fn deref(&self) -> &Self::Target {
            self.0.borrow()
        }
    }

    impl DerefMut for GrpcStatus {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0.borrow_mut()
        }
    }

    impl Debug for GrpcStatus {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let mut builder = f.debug_struct("Status");

            builder.field("code", &self.0.code());

            if !self.0.message().is_empty() {
                builder.field("message", &self.0.message());
            }

            if !self.0.details().is_empty() {
                builder.field("details", &String::from_utf8_lossy(self.0.details()));
            }

            if !self.0.metadata().is_empty() {
                builder.field("metadata", &self.0.metadata());
            }

            builder.field("source", &self.0.source());

            builder.finish()
        }
    }

    impl Display for GrpcStatus {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "status: {:?}, message: {:?}, details: {}, metadata: {:?}",
                self.code(),
                self.message(),
                String::from_utf8_lossy(self.details()),
                self.metadata(),
            )
        }
    }

    impl Error for GrpcStatus {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.0.source().as_ref().map(|err| (&**err) as _)
        }
    }

    impl From<TonicStatus> for GrpcStatus {
        fn from(value: TonicStatus) -> Self {
            Self(value)
        }
    }

    impl From<GrpcStatus> for TonicStatus {
        fn from(value: GrpcStatus) -> Self {
            value.0
        }
    }

    impl<'a> StatusExt<'a> for GrpcStatus {
        fn add_detail(self, detail: ErrorDetail<'a>) -> Self {
            self.0.add_detail(detail).into()
        }
    }
}

use serde::Serialize;
use tonic::{Code, Status};

use crate::infrastructure::grpc_client::GrpcClientError;

/// Safe, stable IPC errors for connection checks and graph requests.
#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[error("{message}")]
pub struct GrpcError {
    pub code: &'static str,
    pub message: &'static str,
}

impl GrpcError {
    /// Creates a safe error without forwarding transport metadata or credentials.
    pub fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }

    /// Marks a result belonging to an obsolete settings or connection generation.
    pub fn superseded() -> Self {
        Self::new("superseded", "This operation was superseded")
    }

    /// Reports a bounded operation that did not finish in time.
    pub fn timeout() -> Self {
        Self::new("timeout", "The gRPC request timed out")
    }

    /// Maps health status failures; unavailable health APIs cannot verify connectivity.
    pub fn health_status(status: Status) -> Self {
        match status.code() {
            Code::NotFound | Code::Unimplemented => Self::new(
                "incompatible_service",
                "The server does not support the required health check",
            ),
            _ => status.into(),
        }
    }

    /// Whether this error requires a new connection check before further graph requests.
    pub fn invalidates_connection(&self) -> bool {
        matches!(
            self.code,
            "authentication_failed" | "unavailable" | "timeout"
        )
    }
}

impl From<Status> for GrpcError {
    fn from(status: Status) -> Self {
        match status.code() {
            Code::Unauthenticated | Code::PermissionDenied => Self::new(
                "authentication_failed",
                "Authentication failed. Check the access token and server access policy",
            ),
            Code::Unavailable => Self::new("unavailable", "The gRPC service is unavailable"),
            Code::DeadlineExceeded => Self::timeout(),
            _ => Self::new("request_failed", "The graph request failed"),
        }
    }
}

impl From<GrpcClientError> for GrpcError {
    fn from(error: GrpcClientError) -> Self {
        match error {
            GrpcClientError::Transport(_) => {
                Self::new("unavailable", "Unable to connect to the gRPC server")
            }
            GrpcClientError::Health(status) => Self::health_status(status),
            GrpcClientError::NotServing => {
                Self::new("not_serving", "The gRPC service is not serving")
            }
            GrpcClientError::Timeout => Self::timeout(),
            _ => Self::new(
                "invalid_configuration",
                "Check the URL, access token, TLS and proxy settings",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_are_safe_and_classified_by_operation() {
        for code in [
            Code::Unauthenticated,
            Code::PermissionDenied,
            Code::Unavailable,
            Code::DeadlineExceeded,
        ] {
            let error = GrpcError::from(Status::new(code, "secret payload"));
            assert!(error.invalidates_connection());
            assert!(
                !serde_json::to_string(&error)
                    .unwrap()
                    .contains("secret payload")
            );
        }
        for code in [
            Code::InvalidArgument,
            Code::ResourceExhausted,
            Code::Internal,
            Code::Cancelled,
        ] {
            assert!(!GrpcError::from(Status::new(code, "private")).invalidates_connection());
        }
        for code in [Code::NotFound, Code::Unimplemented] {
            assert_eq!(
                GrpcError::health_status(Status::new(code, "private")).code,
                "incompatible_service"
            );
        }
    }
}

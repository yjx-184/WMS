use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Unified application error type.
///
/// Each variant carries a business error code and HTTP status code,
/// and serializes to the standard JSON error response:
/// `{"code": <number>, "data": null, "message": "<message>"}`
#[derive(Debug)]
pub enum AppError {
    /// Request parameter validation failure.  HTTP 400 / code 40001.
    Validation(String),
    /// Resource not found.                   HTTP 404 / code 40401.
    NotFound(String),
    /// Business conflict (e.g. duplicate).   HTTP 409 / code 40901.
    Conflict(String),
    /// Business-rule violation (e.g. stock). HTTP 422 / code 42201.
    BusinessRule(String),
    /// Unexpected internal / DB error.       HTTP 500 / code 50001.
    Internal(String),
}

impl AppError {
    fn code(&self) -> u16 {
        match self {
            AppError::Validation(_) => 40001,
            AppError::NotFound(_) => 40401,
            AppError::Conflict(_) => 40901,
            AppError::BusinessRule(_) => 42201,
            AppError::Internal(_) => 50001,
        }
    }

    fn http_status(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::BusinessRule(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "code": self.code(),
            "data": serde_json::Value::Null,
            "message": self.message(),
        }));
        (self.http_status(), body).into_response()
    }
}

impl AppError {
    fn message(&self) -> &str {
        match self {
            AppError::Validation(msg)
            | AppError::NotFound(msg)
            | AppError::Conflict(msg)
            | AppError::BusinessRule(msg)
            | AppError::Internal(msg) => msg,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        // Log the raw sqlx error for debugging, but only expose a generic
        // message to the client to avoid leaking database internals.
        tracing::error!(?err, "sqlx error");
        AppError::Internal("Internal database error".to_string())
    }
}

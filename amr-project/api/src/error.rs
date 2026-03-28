use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("not found")]
    NotFound,

    #[error("conflict: validation failed")]
    Conflict(Vec<String>),

    #[error("rate limited")]
    RateLimited { retry_after: u64 },

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// Attach a request ID to the error response.
    pub fn into_response_with_request_id(self, request_id: Uuid) -> Response {
        let (status, code, message) = self.status_and_message();

        let body = json!({
            "error": {
                "code": code,
                "message": message,
                "request_id": request_id.to_string(),
            }
        });

        (status, Json(body)).into_response()
    }

    fn status_and_message(&self) -> (StatusCode, &'static str, String) {
        match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "unauthorized".into(),
            ),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg.clone()),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".into(),
            ),
            AppError::Conflict(ref conflicts) => (
                StatusCode::CONFLICT,
                "conflict",
                format!("validation failed: {}", conflicts.join("; ")),
            ),
            AppError::RateLimited { retry_after } => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                format!("retry after {}s", retry_after),
            ),
            AppError::Database(e) => {
                tracing::error!("database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal error".into(),
                )
            }
            AppError::Internal(e) => {
                tracing::error!("internal error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal error".into(),
                )
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.status_and_message();

        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

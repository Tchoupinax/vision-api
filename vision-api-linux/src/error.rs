use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not implemented on Linux: {0}")]
    NotImplemented(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    reason: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, reason) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotImplemented(msg) => (StatusCode::NOT_IMPLEMENTED, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        tracing::error!(%reason, "request failed");
        (
            status,
            Json(ErrorBody {
                error: status.canonical_reason().unwrap_or("Error").to_string(),
                reason,
            }),
        )
            .into_response()
    }
}

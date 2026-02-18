use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CultureError {
    #[error("Dynasty not found: {0}")]
    DynastyNotFound(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Cultural conflict: {0}")]
    CulturalConflict(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for CultureError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            CultureError::DynastyNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            CultureError::ElementNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            CultureError::ValidationFailed(msg) => (StatusCode::BAD_REQUEST, msg),
            CultureError::CulturalConflict(msg) => (StatusCode::CONFLICT, msg),
            CultureError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            CultureError::Serde(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            CultureError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, CultureError>;

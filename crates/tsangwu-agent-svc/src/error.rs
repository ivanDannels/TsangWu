use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    NotFound(String),

    #[error("Invalid agent type: {0}")]
    InvalidType(String),

    #[error("Agent is busy: {0}")]
    Busy(String),

    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("PyBridge error: {0}")]
    PyBridge(String),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AgentError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AgentError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AgentError::InvalidType(msg) => (StatusCode::BAD_REQUEST, msg),
            AgentError::Busy(msg) => (StatusCode::CONFLICT, msg),
            AgentError::ExecutionFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AgentError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AgentError::PyBridge(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AgentError::Serde(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AgentError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AgentError>;

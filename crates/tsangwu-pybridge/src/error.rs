use thiserror::Error;

#[derive(Debug, Error)]
pub enum PyBridgeError {
    #[error("Python runtime error: {0}")]
    RuntimeError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, PyBridgeError>;

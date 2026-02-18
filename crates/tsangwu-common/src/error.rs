use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("未找到资源: {0}")]
    NotFound(String),
    #[error("未授权")]
    Unauthorized,
    #[error("权限不足")]
    Forbidden,
    #[error("参数校验失败: {0}")]
    Validation(String),
    #[error("业务冲突: {0}")]
    Conflict(String),
    #[error("限流")]
    RateLimited,
    #[error("内部错误: {0}")]
    Internal(#[from] anyhow::Error),
    #[error("数据库错误: {0}")]
    Database(String),
    #[error("网络错误: {0}")]
    Network(String),
    #[error("超时: {0}")]
    Timeout(String),
    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),
    #[error("外部服务错误: {0}")]
    ExternalService(String),
}

impl AppError {
    pub fn code(&self) -> u32 {
        match self {
            Self::NotFound(_) => 40400,
            Self::Unauthorized => 40100,
            Self::Forbidden => 40300,
            Self::Validation(_) => 42200,
            Self::Conflict(_) => 40900,
            Self::RateLimited => 42900,
            Self::Internal(_) => 50000,
            Self::Database(_) => 50001,
            Self::Network(_) => 50002,
            Self::Timeout(_) => 50003,
            Self::ServiceUnavailable(_) => 50300,
            Self::ExternalService(_) => 50004,
        }
    }

    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout(_) | Self::Network(_) | Self::ServiceUnavailable(_) | Self::Database(_)
        )
    }

    /// 判断错误是否为临时性错误
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::Timeout(_) | Self::Network(_) | Self::ServiceUnavailable(_) | Self::RateLimited
        )
    }

    /// 获取重试延迟（毫秒）
    pub fn retry_delay_ms(&self) -> Option<u64> {
        match self {
            Self::RateLimited => Some(1000),
            Self::Timeout(_) => Some(500),
            Self::Network(_) => Some(300),
            Self::ServiceUnavailable(_) => Some(2000),
            Self::Database(_) => Some(100),
            _ => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Network(_) => StatusCode::BAD_GATEWAY,
            Self::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::ExternalService(_) => StatusCode::BAD_GATEWAY,
        };
        let body = serde_json::json!({
            "code": self.code(),
            "message": self.to_string(),
            "retryable": self.is_retryable(),
            "data": null
        });
        (status, axum::Json(body)).into_response()
    }
}

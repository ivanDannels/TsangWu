pub mod knowledge;
pub mod identify;
pub mod validate;
pub mod matching;

pub use identify::CultureIdentifier;
pub use validate::CultureValidator;
pub use matching::CultureMatcher;

/// 文化适配服务路由
pub fn routes<S: Clone + Send + Sync + 'static>() -> axum::Router<S> {
    use axum::{routing::post, Json};
    use tsangwu_common::{ApiResponse, AppError};

    async fn identify_culture(
        Json(body): Json<serde_json::Value>,
    ) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
        let text = body["text"].as_str().unwrap_or("");
        let identifier = CultureIdentifier::new();
        let tags = identifier.identify(text, &[]).await.map_err(|e| AppError::Internal(e))?;
        Ok(Json(ApiResponse::ok(serde_json::to_value(tags).unwrap())))
    }

    async fn validate_culture(
        Json(body): Json<serde_json::Value>,
    ) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
        let dynasty = body["dynasty"].as_str();
        let validator = CultureValidator::new();
        let result = validator.validate(dynasty, &[], &[]).await.map_err(|e| AppError::Internal(e))?;
        Ok(Json(ApiResponse::ok(serde_json::to_value(result).unwrap())))
    }

    async fn match_resources(
        Json(body): Json<serde_json::Value>,
    ) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
        let labels: Vec<String> = body["labels"].as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let dynasty = body["dynasty"].as_str();
        let matcher = CultureMatcher::new();
        let result = matcher.match_resources(&labels, dynasty).await.map_err(|e| AppError::Internal(e))?;
        Ok(Json(ApiResponse::ok(serde_json::to_value(result).unwrap())))
    }

    axum::Router::new()
        .route("/v1/culture/identify", post(identify_culture))
        .route("/v1/culture/validate", post(validate_culture))
        .route("/v1/culture/match", post(match_resources))
}

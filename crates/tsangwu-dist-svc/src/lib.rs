pub mod platform;

use axum::extract::{Extension, Path};
use axum::{routing::{get, post}, Json};
use tsangwu_auth::Claims;
use tsangwu_common::{ApiResponse, AppError};
use tsangwu_db::DbConn;
use sea_orm::*;

async fn create_distribution(
    Extension(db): Extension<DbConn>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let task_uid_str = body["task_id"].as_str().ok_or(AppError::Validation("缺少 task_id".into()))?;
    let task_uid = uuid::Uuid::parse_str(task_uid_str).map_err(|_| AppError::Validation("无效的 task_id".into()))?;
    let platform = body["platform"].as_str().unwrap_or("douyin");

    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, task_uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;

    let dist_uid = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();
    let model = tsangwu_db::entities::distribution::ActiveModel {
        id: NotSet,
        uid: Set(dist_uid),
        task_id: Set(task.id),
        platform: Set(platform.to_string()),
        status: Set(0), // queued
        result_url: Set(None),
        created_at: Set(now),
    };
    tsangwu_db::repositories::distribution_repo::DistributionRepo::create(&db, model).await
        .map_err(|e| AppError::Internal(e))?;

    // 模拟分发
    let client = platform::MockPlatformClient;
    let result = client.publish(platform, &task_uid_str).await;
    tracing::info!(dist_uid = %dist_uid, platform = %platform, "分发任务已创建");

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "distribution_id": dist_uid.to_string(),
        "status": "queued",
        "platform": platform,
        "mock_result": result,
    }))))
}

async fn get_distribution(
    Extension(db): Extension<DbConn>,
    Path(dist_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let uid = uuid::Uuid::parse_str(&dist_id).map_err(|_| AppError::Validation("无效的分发ID".into()))?;
    let dist = tsangwu_db::repositories::distribution_repo::DistributionRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("分发记录不存在".into()))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "distribution_id": dist.uid.to_string(),
        "platform": dist.platform,
        "status": dist.status,
        "result_url": dist.result_url,
    }))))
}

async fn list_platforms() -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "platforms": [
            {"id": "douyin", "name": "抖音", "enabled": true},
            {"id": "bilibili", "name": "哔哩哔哩", "enabled": true},
            {"id": "xiaohongshu", "name": "小红书", "enabled": true},
            {"id": "wechat_channel", "name": "微信视频号", "enabled": true}
        ]
    }))))
}

pub fn routes<S: Clone + Send + Sync + 'static>() -> axum::Router<S> {
    axum::Router::new()
        .route("/v1/distributions", post(create_distribution))
        .route("/v1/distributions/{dist_id}", get(get_distribution))
        .route("/v1/platforms", get(list_platforms))
}

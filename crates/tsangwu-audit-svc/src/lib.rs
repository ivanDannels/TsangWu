use axum::extract::{Extension, Path};
use axum::{routing::{get, post}, Json};
use sea_orm::*;
use tsangwu_common::{ApiResponse, AppError};
use tsangwu_db::DbConn;

async fn get_review(
    Extension(db): Extension<DbConn>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let task_uid = uuid::Uuid::parse_str(&task_id).map_err(|_| AppError::Validation("无效的任务ID".into()))?;
    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, task_uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;
    let review = tsangwu_db::repositories::review_repo::ReviewRepo::find_by_task(&db, task.id).await
        .map_err(|e| AppError::Internal(e))?;
    match review {
        Some(r) => Ok(Json(ApiResponse::ok(serde_json::json!({
            "id": r.id,
            "task_id": task_id,
            "status": r.status,
            "result": r.result,
            "created_at": r.created_at.to_string(),
        })))),
        None => Ok(Json(ApiResponse::ok(serde_json::json!({"status": "not_submitted"})))),
    }
}

async fn submit_review(
    Extension(db): Extension<DbConn>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let task_uid_str = body["task_id"].as_str().ok_or(AppError::Validation("缺少 task_id".into()))?;
    let task_uid = uuid::Uuid::parse_str(task_uid_str).map_err(|_| AppError::Validation("无效的 task_id".into()))?;
    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, task_uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;

    let now = chrono::Utc::now().fixed_offset();
    // 模拟审核：自动通过
    let model = tsangwu_db::entities::content_review::ActiveModel {
        id: NotSet,
        task_id: Set(task.id),
        status: Set(1), // 1 = passed
        result: Set(Some(serde_json::json!({"auto_review": true, "passed": true, "reason": "模拟审核通过"}))),
        created_at: Set(now),
    };
    let review = tsangwu_db::repositories::review_repo::ReviewRepo::create(&db, model).await
        .map_err(|e| AppError::Internal(e))?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "id": review.id,
        "status": "passed",
        "result": review.result,
    }))))
}

pub fn routes<S: Clone + Send + Sync + 'static>() -> axum::Router<S> {
    axum::Router::new()
        .route("/v1/reviews/{task_id}", get(get_review))
        .route("/v1/reviews", post(submit_review))
}

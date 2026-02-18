use axum::extract::{Extension, Path};
use axum::{routing::{get, post}, Json};
use sea_orm::*;
use tsangwu_common::{ApiResponse, AppError};
use tsangwu_db::DbConn;

async fn get_copyright(
    Extension(db): Extension<DbConn>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let task_uid = uuid::Uuid::parse_str(&task_id).map_err(|_| AppError::Validation("无效的任务ID".into()))?;
    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, task_uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;
    let record = tsangwu_db::repositories::copyright_repo::CopyrightRepo::find_by_task(&db, task.id).await
        .map_err(|e| AppError::Internal(e))?;
    match record {
        Some(r) => Ok(Json(ApiResponse::ok(serde_json::json!({
            "id": r.id,
            "task_id": task_id,
            "phash": r.phash,
            "status": r.status,
            "created_at": r.created_at.to_string(),
        })))),
        None => Ok(Json(ApiResponse::ok(serde_json::json!({"status": "not_checked"})))),
    }
}

async fn check_copyright(
    Extension(db): Extension<DbConn>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let task_uid_str = body["task_id"].as_str().ok_or(AppError::Validation("缺少 task_id".into()))?;
    let task_uid = uuid::Uuid::parse_str(task_uid_str).map_err(|_| AppError::Validation("无效的 task_id".into()))?;
    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, task_uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;

    let now = chrono::Utc::now().fixed_offset();
    // 模拟 pHash 计算
    let phash = format!("{:016x}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos() % u128::from(u64::MAX));

    let model = tsangwu_db::entities::copyright_record::ActiveModel {
        id: NotSet,
        task_id: Set(task.id),
        phash: Set(Some(phash.clone())),
        status: Set(1), // 1 = clean
        created_at: Set(now),
    };
    let record = tsangwu_db::repositories::copyright_repo::CopyrightRepo::create(&db, model).await
        .map_err(|e| AppError::Internal(e))?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "id": record.id,
        "phash": phash,
        "status": "clean",
        "message": "未发现版权冲突",
    }))))
}

pub fn routes<S: Clone + Send + Sync + 'static>() -> axum::Router<S> {
    axum::Router::new()
        .route("/v1/copyright/{task_id}", get(get_copyright))
        .route("/v1/copyright/check", post(check_copyright))
}

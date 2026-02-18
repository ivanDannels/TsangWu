use axum::extract::{Extension, Path};
use axum::Json;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use tsangwu_auth::Claims;
use tsangwu_common::{ApiResponse, AppError, PageParams};
use tsangwu_db::DbConn;

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub scene_type: String,
    pub mode: String,
    pub input: GenerateInput,
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct GenerateInput {
    pub prompt: String,
    pub reference_images: Option<Vec<String>>,
    pub template_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub task_id: String,
    pub status: String,
    pub ws_channel: String,
}

#[derive(Debug, Serialize)]
pub struct TaskStatusResponse {
    pub task_id: String,
    pub status: String,
    pub progress: i16,
    pub error_msg: Option<String>,
}

impl From<tsangwu_db::entities::generation_task::Model> for TaskStatusResponse {
    fn from(m: tsangwu_db::entities::generation_task::Model) -> Self {
        let status = match m.status {
            0 => "pending", 1 => "queued", 2 => "running",
            3 => "completed", 4 => "failed", 5 => "cancelled",
            _ => "unknown",
        };
        Self {
            task_id: m.uid.to_string(),
            status: status.into(),
            progress: m.progress.unwrap_or(0),
            error_msg: m.error_msg,
        }
    }
}

pub async fn submit_generation(
    Extension(db): Extension<DbConn>,
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<String>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<ApiResponse<GenerateResponse>>, AppError> {
    let user_uid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&db, user_uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::Unauthorized)?;

    let proj_uid = uuid::Uuid::parse_str(&project_id).map_err(|_| AppError::Validation("无效的项目ID".into()))?;
    let project = tsangwu_db::repositories::project_repo::ProjectRepo::find_by_uid(&db, proj_uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("项目不存在".into()))?;

    let task_uid = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();
    let input_params = serde_json::json!({
        "scene_type": req.scene_type,
        "mode": req.mode,
        "prompt": req.input.prompt,
        "reference_images": req.input.reference_images,
        "template_id": req.input.template_id,
        "config": req.config,
    });

    let task_model = tsangwu_db::entities::generation_task::ActiveModel {
        id: NotSet,
        uid: Set(task_uid),
        project_id: Set(project.id),
        user_id: Set(user.id),
        task_type: Set(0),
        priority: Set(0),
        status: Set(1), // queued
        input_params: Set(input_params),
        output_urls: Set(None),
        progress: Set(Some(0)),
        error_msg: Set(None),
        started_at: Set(None),
        completed_at: Set(None),
        created_at: Set(now),
    };

    tsangwu_db::repositories::task_repo::TaskRepo::create(&db, task_model).await
        .map_err(|e| AppError::Internal(e))?;

    Ok(Json(ApiResponse::ok(GenerateResponse {
        task_id: task_uid.to_string(),
        status: "queued".into(),
        ws_channel: format!("ws://localhost:8080/ws/tasks/{}/progress", task_uid),
    })))
}

pub async fn get_task(
    Extension(db): Extension<DbConn>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiResponse<TaskStatusResponse>>, AppError> {
    let uid = uuid::Uuid::parse_str(&task_id).map_err(|_| AppError::Validation("无效的任务ID".into()))?;
    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;
    Ok(Json(ApiResponse::ok(TaskStatusResponse::from(task))))
}

pub async fn get_result(
    Extension(db): Extension<DbConn>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let uid = uuid::Uuid::parse_str(&task_id).map_err(|_| AppError::Validation("无效的任务ID".into()))?;
    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;
    if task.status != 3 {
        return Err(AppError::Validation("任务尚未完成".into()));
    }
    Ok(Json(ApiResponse::ok(task.output_urls.unwrap_or(serde_json::json!({})))))
}

pub async fn cancel(
    Extension(db): Extension<DbConn>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let uid = uuid::Uuid::parse_str(&task_id).map_err(|_| AppError::Validation("无效的任务ID".into()))?;
    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;
    tsangwu_db::repositories::task_repo::TaskRepo::update_status(&db, task.id, 5).await
        .map_err(|e| AppError::Internal(e))?;
    Ok(Json(ApiResponse::ok(())))
}

pub async fn retry(
    Extension(db): Extension<DbConn>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiResponse<GenerateResponse>>, AppError> {
    let uid = uuid::Uuid::parse_str(&task_id).map_err(|_| AppError::Validation("无效的任务ID".into()))?;
    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;
    tsangwu_db::repositories::task_repo::TaskRepo::update_status(&db, task.id, 1).await
        .map_err(|e| AppError::Internal(e))?;
    tsangwu_db::repositories::task_repo::TaskRepo::update_progress(&db, task.id, 0).await
        .map_err(|e| AppError::Internal(e))?;
    Ok(Json(ApiResponse::ok(GenerateResponse {
        task_id: task.uid.to_string(),
        status: "queued".into(),
        ws_channel: format!("ws://localhost:8080/ws/tasks/{}/progress", task.uid),
    })))
}

pub async fn list_tasks(
    Extension(db): Extension<DbConn>,
    Extension(claims): Extension<Claims>,
    axum::extract::Query(params): axum::extract::Query<PageParams>,
) -> Result<Json<ApiResponse<tsangwu_common::PageResult<TaskStatusResponse>>>, AppError> {
    let user_uid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&db, user_uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::Unauthorized)?;
    let (items, total) = tsangwu_db::repositories::task_repo::TaskRepo::list_by_user(&db, user.id, &params).await
        .map_err(|e| AppError::Internal(e))?;
    let resp: Vec<TaskStatusResponse> = items.into_iter().map(TaskStatusResponse::from).collect();
    Ok(Json(ApiResponse::ok(tsangwu_common::PageResult::new(resp, total, &params))))
}

pub async fn export(
    Extension(db): Extension<DbConn>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let uid = uuid::Uuid::parse_str(&task_id).map_err(|_| AppError::Validation("无效的任务ID".into()))?;
    let task = tsangwu_db::repositories::task_repo::TaskRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::NotFound("任务不存在".into()))?;
    if task.status != 3 {
        return Err(AppError::Validation("任务尚未完成，无法导出".into()));
    }
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "task_id": task.uid.to_string(),
        "output_urls": task.output_urls,
    }))))
}

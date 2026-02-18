use axum::extract::{Extension, Path};
use axum::Json;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tsangwu_auth::Claims;
use tsangwu_common::{ApiResponse, AppError, PageParams};
use tsangwu_db::DbConn;

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub title: String,
    pub scene_type: i16,
    pub user_mode: i16,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub title: Option<String>,
    pub status: Option<i16>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub uid: String,
    pub title: String,
    pub scene_type: i16,
    pub status: i16,
}

impl From<tsangwu_db::entities::project::Model> for ProjectResponse {
    fn from(m: tsangwu_db::entities::project::Model) -> Self {
        Self {
            uid: m.uid.to_string(),
            title: m.title,
            scene_type: m.scene_type,
            status: m.status,
        }
    }
}

pub async fn create(
    Extension(db): Extension<DbConn>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<ApiResponse<ProjectResponse>>, AppError> {
    let uid_parsed = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&db, uid_parsed).await
        .map_err(|e| AppError::Internal(e))?
        .ok_or(AppError::Unauthorized)?;

    let now = chrono::Utc::now().fixed_offset();
    let project_uid = uuid::Uuid::new_v4();
    let model = tsangwu_db::entities::project::ActiveModel {
        id: NotSet,
        uid: Set(project_uid),
        owner_id: Set(user.id),
        org_id: Set(None),
        title: Set(req.title),
        scene_type: Set(req.scene_type),
        user_mode: Set(req.user_mode),
        status: Set(0),
        config: Set(req.config.unwrap_or(serde_json::json!({}))),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let project = tsangwu_db::repositories::project_repo::ProjectRepo::create(&db, model).await
        .map_err(|e| AppError::Internal(e))?;
    Ok(Json(ApiResponse::ok(ProjectResponse::from(project))))
}

pub async fn list(
    Extension(db): Extension<DbConn>,
    Extension(claims): Extension<Claims>,
    axum::extract::Query(params): axum::extract::Query<PageParams>,
) -> Result<Json<ApiResponse<tsangwu_common::PageResult<ProjectResponse>>>, AppError> {
    let uid_parsed = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&db, uid_parsed).await
        .map_err(|e| AppError::Internal(e))?
        .ok_or(AppError::Unauthorized)?;

    let (items, total) = tsangwu_db::repositories::project_repo::ProjectRepo::list_by_owner(&db, user.id, &params).await
        .map_err(|e| AppError::Internal(e))?;
    let resp: Vec<ProjectResponse> = items.into_iter().map(ProjectResponse::from).collect();
    Ok(Json(ApiResponse::ok(tsangwu_common::PageResult::new(resp, total, &params))))
}

pub async fn get(
    Extension(db): Extension<DbConn>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ProjectResponse>>, AppError> {
    let uid = uuid::Uuid::parse_str(&id).map_err(|_| AppError::Validation("无效的项目ID".into()))?;
    let project = tsangwu_db::repositories::project_repo::ProjectRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?
        .ok_or(AppError::NotFound("项目不存在".into()))?;
    Ok(Json(ApiResponse::ok(ProjectResponse::from(project))))
}

pub async fn update(
    Extension(db): Extension<DbConn>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ApiResponse<ProjectResponse>>, AppError> {
    let uid = uuid::Uuid::parse_str(&id).map_err(|_| AppError::Validation("无效的项目ID".into()))?;
    let project = tsangwu_db::repositories::project_repo::ProjectRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?
        .ok_or(AppError::NotFound("项目不存在".into()))?;

    let now = chrono::Utc::now().fixed_offset();
    let mut active: tsangwu_db::entities::project::ActiveModel = project.into();
    if let Some(title) = req.title {
        active.title = Set(title);
    }
    if let Some(status) = req.status {
        active.status = Set(status);
    }
    if let Some(config) = req.config {
        active.config = Set(config);
    }
    active.updated_at = Set(now);

    let updated = tsangwu_db::repositories::project_repo::ProjectRepo::update(&db, active).await
        .map_err(|e| AppError::Internal(e))?;
    Ok(Json(ApiResponse::ok(ProjectResponse::from(updated))))
}

pub async fn remove(
    Extension(db): Extension<DbConn>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let uid = uuid::Uuid::parse_str(&id).map_err(|_| AppError::Validation("无效的项目ID".into()))?;
    let project = tsangwu_db::repositories::project_repo::ProjectRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?
        .ok_or(AppError::NotFound("项目不存在".into()))?;
    tsangwu_db::repositories::project_repo::ProjectRepo::delete(&db, project.id).await
        .map_err(|e| AppError::Internal(e))?;
    Ok(Json(ApiResponse::ok(())))
}

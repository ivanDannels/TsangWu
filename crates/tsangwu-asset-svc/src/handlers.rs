use axum::extract::{Extension, Path};
use axum::Json;
use serde::{Deserialize, Serialize};
use tsangwu_auth::Claims;
use tsangwu_common::{ApiResponse, AppError};
use tsangwu_db::DbConn;

#[derive(Debug, Serialize)]
pub struct AssetResponse {
    pub asset_uid: String,
    pub name: String,
    pub category: String,
    pub tags: Vec<String>,
    pub thumbnail_url: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub category: Option<String>,
    pub dynasty: Option<String>,
}

pub async fn list(
    Extension(_db): Extension<DbConn>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<AssetResponse>>>, AppError> {
    // 素材列表 - 简化实现返回空列表
    Ok(Json(ApiResponse::ok(vec![])))
}

pub async fn get(
    Extension(_db): Extension<DbConn>,
    Path(_id): Path<String>,
) -> Result<Json<ApiResponse<AssetResponse>>, AppError> {
    Err(AppError::NotFound("素材不存在".into()))
}

pub async fn remove(
    Extension(_db): Extension<DbConn>,
    Path(_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    Ok(Json(ApiResponse::ok(())))
}

pub async fn semantic_search(
    Extension(_db): Extension<DbConn>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<ApiResponse<Vec<AssetResponse>>>, AppError> {
    tracing::info!(query = %req.query, "素材语义搜索");
    // 模拟语义搜索：返回基于关键词的模拟结果
    let mut results = vec![];
    if !req.query.is_empty() {
        results.push(AssetResponse {
            asset_uid: uuid::Uuid::new_v4().to_string(),
            name: format!("搜索结果: {}", req.query),
            category: req.category.unwrap_or_else(|| "scene".into()),
            tags: vec![req.query.clone()],
            thumbnail_url: String::new(),
        });
    }
    Ok(Json(ApiResponse::ok(results)))
}

pub async fn upload(
    Extension(_db): Extension<DbConn>,
    Extension(_claims): Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<AssetResponse>>, AppError> {
    let file_name = body["name"].as_str().unwrap_or("unnamed").to_string();
    let asset_uid = uuid::Uuid::new_v4().to_string();
    tracing::info!(asset_uid = %asset_uid, name = %file_name, "素材上传完成");

    Ok(Json(ApiResponse::ok(AssetResponse {
        asset_uid,
        name: file_name,
        category: "user_upload".into(),
        tags: vec![],
        thumbnail_url: String::new(),
    })))
}

pub async fn favorite(
    Extension(_db): Extension<DbConn>,
    Path(_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    Ok(Json(ApiResponse::ok(())))
}

pub async fn list_favorites(
    Extension(_db): Extension<DbConn>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<AssetResponse>>>, AppError> {
    Ok(Json(ApiResponse::ok(vec![])))
}

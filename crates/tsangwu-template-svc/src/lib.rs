use axum::extract::Extension;
use axum::{routing::get, Router};
use tsangwu_common::{ApiResponse, AppError, PageParams};
use axum::Json;
use serde::Serialize;
use tsangwu_db::DbConn;

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: String,
    pub name: String,
    pub scene_type: String,
    pub style: String,
    pub preview_url: String,
    pub tags: Vec<String>,
}

async fn list(
    Extension(_db): Extension<DbConn>,
    axum::extract::Query(_params): axum::extract::Query<PageParams>,
) -> Result<Json<ApiResponse<Vec<TemplateResponse>>>, AppError> {
    // 模拟模板数据
    let templates = vec![
        TemplateResponse {
            id: "tpl_001".into(), name: "唐风仕女".into(),
            scene_type: "image".into(), style: "工笔".into(),
            preview_url: String::new(), tags: vec!["唐代".into(), "仕女".into()],
        },
        TemplateResponse {
            id: "tpl_002".into(), name: "宋代山水".into(),
            scene_type: "image".into(), style: "水墨".into(),
            preview_url: String::new(), tags: vec!["宋代".into(), "山水".into()],
        },
        TemplateResponse {
            id: "tpl_003".into(), name: "明清宫廷".into(),
            scene_type: "short_drama".into(), style: "写实".into(),
            preview_url: String::new(), tags: vec!["明代".into(), "清代".into(), "宫廷".into()],
        },
    ];
    Ok(Json(ApiResponse::ok(templates)))
}

async fn recommend(
    Extension(_db): Extension<DbConn>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<TemplateResponse>>>, AppError> {
    let _tags: Vec<&str> = params.get("tags").map(|t| t.split(',').collect()).unwrap_or_default();
    // 模拟推荐
    let templates = vec![
        TemplateResponse {
            id: "tpl_001".into(), name: "唐风仕女".into(),
            scene_type: "image".into(), style: "工笔".into(),
            preview_url: String::new(), tags: vec!["唐代".into(), "仕女".into()],
        },
    ];
    Ok(Json(ApiResponse::ok(templates)))
}

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/v1/templates", get(list))
        .route("/v1/templates/recommend", get(recommend))
}

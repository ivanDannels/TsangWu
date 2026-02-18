use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{error::*, service::CultureService, types::*};

pub fn routes(service: Arc<CultureService>) -> Router {
    Router::new()
        .route("/dynasties", post(create_dynasty).get(list_dynasties))
        .route("/dynasties/:id", get(get_dynasty))
        .route("/dynasties/:id/elements", get(list_elements))
        .route("/dynasties/:id/knowledge-graph", get(get_knowledge_graph))
        .route("/elements", post(create_element))
        .route("/elements/:id", get(get_element))
        .route("/validate", post(validate_content))
        .route("/recommend", post(recommend_elements))
        .with_state(service)
}

async fn create_dynasty(
    State(service): State<Arc<CultureService>>,
    Json(req): Json<CreateDynastyRequest>,
) -> Result<Json<Dynasty>> {
    let dynasty = service.create_dynasty(req).await?;
    Ok(Json(dynasty))
}

async fn get_dynasty(
    State(service): State<Arc<CultureService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Dynasty>> {
    let dynasty = service.get_dynasty(id).await?;
    Ok(Json(dynasty))
}

async fn list_dynasties(
    State(service): State<Arc<CultureService>>,
) -> Result<Json<Vec<Dynasty>>> {
    let dynasties = service.list_dynasties().await?;
    Ok(Json(dynasties))
}

async fn create_element(
    State(service): State<Arc<CultureService>>,
    Json(req): Json<CreateElementRequest>,
) -> Result<Json<CultureElement>> {
    let element = service.create_element(req).await?;
    Ok(Json(element))
}

async fn get_element(
    State(service): State<Arc<CultureService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CultureElement>> {
    let element = service.get_element(id).await?;
    Ok(Json(element))
}

#[derive(Deserialize)]
struct ListElementsQuery {
    element_type: Option<ElementType>,
}

async fn list_elements(
    State(service): State<Arc<CultureService>>,
    Path(dynasty_id): Path<Uuid>,
    Query(query): Query<ListElementsQuery>,
) -> Result<Json<Vec<CultureElement>>> {
    let elements = service.list_elements(dynasty_id, query.element_type).await?;
    Ok(Json(elements))
}

async fn validate_content(
    State(service): State<Arc<CultureService>>,
    Json(req): Json<ValidateContentRequest>,
) -> Result<Json<ValidateContentResponse>> {
    let response = service.validate_content(req).await?;
    Ok(Json(response))
}

async fn recommend_elements(
    State(service): State<Arc<CultureService>>,
    Json(req): Json<RecommendElementsRequest>,
) -> Result<Json<RecommendElementsResponse>> {
    let response = service.recommend_elements(req).await?;
    Ok(Json(response))
}

async fn get_knowledge_graph(
    State(service): State<Arc<CultureService>>,
    Path(dynasty_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let (nodes, relations) = service.get_knowledge_graph(dynasty_id).await?;
    Ok(Json(serde_json::json!({
        "nodes": nodes,
        "relations": relations
    })))
}

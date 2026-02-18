use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tsangwu_app::AppState;
use tsangwu_auth::Claims;
use uuid::Uuid;

use crate::{error::*, service::AgentService, types::*};

#[derive(Deserialize)]
struct ListTasksQuery {
    limit: Option<i32>,
}

/// Agent服务路由（需要认证）
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/agents", post(create_agent))
        .route("/v1/agents/{id}", get(get_agent))
        .route("/v1/agents/project/{project_id}", get(list_agents))
        .route("/v1/agents/{id}/execute", post(execute_task))
        .route("/v1/agents/{id}/tasks", get(list_agent_tasks))
        .route("/v1/agents/{id}/tasks/{task_id}", get(get_task))
        .route("/v1/agents/{id}/tasks/{task_id}/cancel", delete(cancel_task))
        .route("/v1/agents/{id}/tasks/{task_id}/retry", post(retry_task))
}

/// 从AppState获取AgentService
fn get_service(state: &Arc<AppState>) -> AgentService {
    AgentService::new(state.db.pg_pool(), Arc::clone(&state.pybridge))
}

/// 创建Agent（需要认证）
async fn create_agent(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<Agent>> {
    tracing::info!(user_id = %claims.sub, "创建Agent请求");
    let service = get_service(&state);
    let agent = service.create_agent(req).await?;
    Ok(Json(agent))
}

/// 获取Agent详情（需要认证）
async fn get_agent(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>> {
    let service = get_service(&state);
    let agent = service.get_agent(id).await?;
    Ok(Json(agent))
}

/// 列出项目的所有Agent（需要认证）
async fn list_agents(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Agent>>> {
    let service = get_service(&state);
    let agents = service.list_agents(project_id).await?;
    Ok(Json(agents))
}

/// 执行任务（需要认证）
async fn execute_task(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExecuteTaskRequest>,
) -> Result<Json<AgentTask>> {
    tracing::info!(user_id = %claims.sub, agent_id = %id, "执行Agent任务");
    let service = get_service(&state);
    let task = service.execute_task(id, req).await?;
    Ok(Json(task))
}

/// 获取任务详情（需要认证）
async fn get_task(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((_agent_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<AgentTask>> {
    let service = get_service(&state);
    let task = service.get_task(task_id).await?;
    Ok(Json(task))
}

/// 列出Agent的所有任务（需要认证）
async fn list_agent_tasks(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(agent_id): Path<Uuid>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<Vec<AgentTask>>> {
    let service = get_service(&state);
    let tasks = service.list_tasks(agent_id, query.limit).await?;
    Ok(Json(tasks))
}

/// 取消任务（需要认证）
async fn cancel_task(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((_agent_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!(user_id = %claims.sub, task_id = %task_id, "取消任务");
    let service = get_service(&state);
    service.cancel_task(task_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Task cancelled successfully"
    })))
}

/// 重试失败的任务（需要认证）
async fn retry_task(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((_agent_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<AgentTask>> {
    tracing::info!(user_id = %claims.sub, task_id = %task_id, "重试任务");
    let service = get_service(&state);
    let task = service.retry_task(task_id).await?;
    Ok(Json(task))
}

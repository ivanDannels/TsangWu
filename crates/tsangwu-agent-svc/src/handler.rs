use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{error::*, service::AgentService, types::*};

pub fn routes(service: Arc<AgentService>) -> Router {
    Router::new()
        .route("/agents", post(create_agent))
        .route("/agents/:id", get(get_agent))
        .route("/agents/project/:project_id", get(list_agents))
        .route("/agents/:id/execute", post(execute_task))
        .route("/agents/:id/tasks", get(list_agent_tasks))
        .route("/agents/:id/tasks/:task_id", get(get_task))
        .route("/agents/:id/tasks/:task_id/cancel", delete(cancel_task))
        .route("/agents/:id/tasks/:task_id/retry", post(retry_task))
        .with_state(service)
}

#[derive(Deserialize)]
struct ListTasksQuery {
    limit: Option<i32>,
}

async fn create_agent(
    State(service): State<Arc<AgentService>>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<Agent>> {
    let agent = service.create_agent(req).await?;
    Ok(Json(agent))
}

async fn get_agent(
    State(service): State<Arc<AgentService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>> {
    let agent = service.get_agent(id).await?;
    Ok(Json(agent))
}

async fn list_agents(
    State(service): State<Arc<AgentService>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Agent>>> {
    let agents = service.list_agents(project_id).await?;
    Ok(Json(agents))
}

async fn execute_task(
    State(service): State<Arc<AgentService>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExecuteTaskRequest>,
) -> Result<Json<AgentTask>> {
    let task = service.execute_task(id, req).await?;
    Ok(Json(task))
}

async fn get_task(
    State(service): State<Arc<AgentService>>,
    Path((_agent_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<AgentTask>> {
    let task = service.get_task(task_id).await?;
    Ok(Json(task))
}

/// 列出Agent的所有任务
async fn list_agent_tasks(
    State(service): State<Arc<AgentService>>,
    Path(agent_id): Path<Uuid>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<Vec<AgentTask>>> {
    let tasks = service.list_tasks(agent_id, query.limit).await?;
    Ok(Json(tasks))
}

/// 取消任务
async fn cancel_task(
    State(service): State<Arc<AgentService>>,
    Path((_agent_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>> {
    service.cancel_task(task_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Task cancelled successfully"
    })))
}

/// 重试失败的任务
async fn retry_task(
    State(service): State<Arc<AgentService>>,
    Path((_agent_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<AgentTask>> {
    let task = service.retry_task(task_id).await?;
    Ok(Json(task))
}

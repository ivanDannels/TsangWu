use axum::{routing::{get, post}, Router};

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/v1/projects/{id}/generate", post(super::handlers::submit_generation))
        .route("/v1/tasks", get(super::handlers::list_tasks))
        .route("/v1/tasks/{task_id}", get(super::handlers::get_task))
        .route("/v1/tasks/{task_id}/result", get(super::handlers::get_result))
        .route("/v1/tasks/{task_id}/cancel", post(super::handlers::cancel))
        .route("/v1/tasks/{task_id}/retry", post(super::handlers::retry))
        .route("/v1/tasks/{task_id}/export", post(super::handlers::export))
}

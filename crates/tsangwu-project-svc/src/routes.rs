use axum::{routing::{get, post}, Router};

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/v1/projects", post(super::handlers::create).get(super::handlers::list))
        .route("/v1/projects/{id}", get(super::handlers::get).put(super::handlers::update).delete(super::handlers::remove))
}

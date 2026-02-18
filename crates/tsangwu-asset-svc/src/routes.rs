use axum::{routing::{get, post}, Router};

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/v1/assets", get(super::handlers::list))
        .route("/v1/assets/{id}", get(super::handlers::get).delete(super::handlers::remove))
        .route("/v1/assets/search", post(super::handlers::semantic_search))
        .route("/v1/assets/upload", post(super::handlers::upload))
        .route("/v1/assets/{id}/favorite", post(super::handlers::favorite))
        .route("/v1/assets/favorites", get(super::handlers::list_favorites))
}

use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use tsangwu_app::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/auth/register", post(super::handlers::register))
        .route("/v1/auth/login", post(super::handlers::login))
        .route("/v1/users/me", get(super::handlers::get_me))
}

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: tsangwu_db::DbConn,
    pub jwt: Arc<tsangwu_auth::JwtManager>,
    pub pybridge: Arc<tsangwu_pybridge::PyBridge>,
}

impl AppState {
    pub fn new(db: tsangwu_db::DbConn, jwt: Arc<tsangwu_auth::JwtManager>) -> Self {
        let pybridge = Arc::new(
            tsangwu_pybridge::PyBridge::new(2)
                .expect("Failed to initialize PyBridge")
        );
        Self { db, jwt, pybridge }
    }

    pub fn with_pybridge(db: tsangwu_db::DbConn, jwt: Arc<tsangwu_auth::JwtManager>, pybridge: Arc<tsangwu_pybridge::PyBridge>) -> Self {
        Self { db, jwt, pybridge }
    }
}

use axum::extract::Extension;
use axum::{routing::post, Router};
use std::sync::Arc;
use tsangwu_auth::{Claims, JwtManager};
use tsangwu_common::{ApiResponse, AppError};
use axum::Json;
use serde::Deserialize;
use tsangwu_db::DbConn;

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

async fn refresh_token(
    Extension(db): Extension<DbConn>,
    Extension(jwt): Extension<Arc<JwtManager>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    // 验证 refresh token
    let old_claims = jwt.verify_access_token(&req.refresh_token)
        .map_err(|_| AppError::Unauthorized)?;

    // 查询用户确认存在
    let uid = uuid::Uuid::parse_str(&old_claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?
        .ok_or(AppError::Unauthorized)?;

    // 获取用户角色
    let roles = tsangwu_db::repositories::role_repo::RoleRepo::find_role_codes_by_user(&db, user.id).await
        .unwrap_or_else(|_| vec!["user".into()]);

    // 签发新 access token
    let claims = Claims {
        sub: user.uid.to_string(),
        roles,
        org_id: None,
        plan: "free".into(),
        exp: 0,
        iat: 0,
    };
    let access_token = jwt.issue_access_token(&claims).map_err(|e| AppError::Internal(e))?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "access_token": access_token,
    }))))
}

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/v1/auth/refresh", post(refresh_token))
}

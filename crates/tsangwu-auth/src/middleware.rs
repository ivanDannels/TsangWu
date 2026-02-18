use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tsangwu_common::AppError;
use tsangwu_db::DbConn;
use tsangwu_db::entities::api_key;
use sea_orm::*;

use crate::jwt::{Claims, JwtManager};

/// Axum 认证中间件：从 Authorization header 提取并验证 JWT
pub async fn auth_middleware(
    State(jwt): State<Arc<JwtManager>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = jwt
        .verify_access_token(token)
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// API Key 认证中间件（开放 API）
pub async fn api_key_auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or(AppError::Unauthorized)?;

    let db = req
        .extensions()
        .get::<DbConn>()
        .cloned()
        .ok_or(AppError::Internal(anyhow::anyhow!("数据库连接未注入")))?;

    let record = api_key::Entity::find()
        .filter(api_key::Column::KeyHash.eq(&api_key))
        .one(&db.conn)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .ok_or(AppError::Unauthorized)?;

    // status = 1 表示 active
    if record.status != 1 {
        return Err(AppError::Unauthorized);
    }

    let claims = Claims {
        sub: record.user_id.to_string(),
        roles: vec![],
        org_id: None,
        plan: String::new(),
        exp: usize::MAX,
        iat: 0,
    };

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

use axum::extract::{Extension, State};
use axum::Json;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tsangwu_app::AppState;
use tsangwu_auth::Claims;
use tsangwu_common::{ApiResponse, AppError};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub nickname: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_uid: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub account: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub uid: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub account_type: i16,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    // 校验
    if req.phone.is_none() && req.email.is_none() {
        return Err(AppError::Validation("手机号或邮箱至少填一项".into()));
    }
    if req.password.len() < 6 {
        return Err(AppError::Validation("密码至少6位".into()));
    }

    // 查重
    if let Some(ref email) = req.email {
        if tsangwu_db::repositories::user_repo::UserRepo::find_by_email(&state.db, email).await?.is_some() {
            return Err(AppError::Conflict("邮箱已注册".into()));
        }
    }
    if let Some(ref phone) = req.phone {
        if tsangwu_db::repositories::user_repo::UserRepo::find_by_phone(&state.db, phone).await?.is_some() {
            return Err(AppError::Conflict("手机号已注册".into()));
        }
    }

    // 哈希密码
    let password_hash = tsangwu_auth::password::hash_password(&req.password)
        .map_err(|e| AppError::Internal(e))?;

    let uid = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().fixed_offset();
    let user_model = tsangwu_db::entities::user::ActiveModel {
        id: NotSet,
        uid: Set(uid),
        phone: Set(req.phone),
        email: Set(req.email),
        nickname: Set(req.nickname),
        avatar_url: Set(None),
        password_hash: Set(Some(password_hash)),
        account_type: Set(0),
        status: Set(1),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let user = tsangwu_db::repositories::user_repo::UserRepo::create(&state.db, user_model).await
        .map_err(|e| AppError::Internal(e))?;

    // 签发 JWT
    let claims = Claims {
        sub: user.uid.to_string(),
        roles: vec!["user".into()],
        org_id: None,
        plan: "free".into(),
        exp: 0,
        iat: 0,
    };
    let access_token = state.jwt.issue_access_token(&claims).map_err(|e| AppError::Internal(e))?;
    let refresh_token = state.jwt.issue_refresh_token(&user.uid.to_string()).map_err(|e| AppError::Internal(e))?;

    Ok(Json(ApiResponse::ok(AuthResponse {
        access_token,
        refresh_token,
        user_uid: user.uid.to_string(),
    })))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    tracing::info!(account = %req.account, "用户登录");

    // 按邮箱或手机号查找
    let user = if req.account.contains('@') {
        tsangwu_db::repositories::user_repo::UserRepo::find_by_email(&state.db, &req.account).await
    } else {
        tsangwu_db::repositories::user_repo::UserRepo::find_by_phone(&state.db, &req.account).await
    }.map_err(|e| AppError::Internal(e))?
     .ok_or(AppError::Validation("账号或密码错误".into()))?;

    // 验证密码
    let hash = user.password_hash.as_deref().ok_or(AppError::Validation("账号或密码错误".into()))?;
    let valid = tsangwu_auth::password::verify_password(&req.password, hash)
        .map_err(|e| AppError::Internal(e))?;
    if !valid {
        return Err(AppError::Validation("账号或密码错误".into()));
    }

    let claims = Claims {
        sub: user.uid.to_string(),
        roles: vec!["user".into()],
        org_id: None,
        plan: "free".into(),
        exp: 0,
        iat: 0,
    };
    let access_token = state.jwt.issue_access_token(&claims).map_err(|e| AppError::Internal(e))?;
    let refresh_token = state.jwt.issue_refresh_token(&user.uid.to_string()).map_err(|e| AppError::Internal(e))?;

    Ok(Json(ApiResponse::ok(AuthResponse {
        access_token,
        refresh_token,
        user_uid: user.uid.to_string(),
    })))
}

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<UserInfo>>, AppError> {
    let uid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&state.db, uid).await
        .map_err(|e| AppError::Internal(e))?
        .ok_or(AppError::NotFound("用户不存在".into()))?;

    Ok(Json(ApiResponse::ok(UserInfo {
        uid: user.uid.to_string(),
        nickname: user.nickname,
        avatar_url: user.avatar_url,
        account_type: user.account_type,
    })))
}

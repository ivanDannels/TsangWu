use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tsangwu_common::AppError;

/// 权限检查中间件工厂
pub fn require_permission(permission: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    let perm = permission.to_string();
    move |req: Request, next: Next| {
        let perm = perm.clone();
        Box::pin(async move {
            // 从扩展中获取 Claims
            let claims = req.extensions().get::<crate::Claims>().cloned();

            if let Some(claims) = claims {
                // 检查权限
                if has_permission(&claims, &perm).await {
                    Ok(next.run(req).await)
                } else {
                    Err(StatusCode::FORBIDDEN)
                }
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        })
    }
}

/// 角色检查中间件工厂
pub fn require_role(role: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    let required_role = role.to_string();
    move |req: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            let claims = req.extensions().get::<crate::Claims>().cloned();

            if let Some(claims) = claims {
                if claims.roles.contains(&required_role) {
                    Ok(next.run(req).await)
                } else {
                    Err(StatusCode::FORBIDDEN)
                }
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        })
    }
}

/// 检查用户是否拥有指定权限
async fn has_permission(claims: &crate::Claims, permission: &str) -> bool {
    // platform_admin 拥有所有权限
    if claims.roles.contains(&"platform_admin".to_string()) {
        return true;
    }

    // 这里可以扩展为查询数据库获取角色权限
    // 目前简化为基于角色的硬编码权限
    match permission {
        "user:read" => claims.roles.iter().any(|r| r == "user" || r == "admin"),
        "user:write" => claims.roles.contains(&"admin".to_string()),
        "project:read" => claims.roles.iter().any(|r| r == "user" || r == "admin"),
        "project:write" => claims.roles.iter().any(|r| r == "user" || r == "admin"),
        "admin:*" => claims.roles.contains(&"admin".to_string()),
        _ => false,
    }
}

/// 权限宏：用于在 handler 中快速检查权限
#[macro_export]
macro_rules! require_perm {
    ($claims:expr, $permission:expr) => {
        if !$crate::rbac::check_permission(&$claims, $permission).await {
            return Err($crate::AppError::Forbidden);
        }
    };
}

/// 角色宏：用于在 handler 中快速检查角色
#[macro_export]
macro_rules! require_role {
    ($claims:expr, $role:expr) => {
        if !$claims.roles.contains(&$role.to_string()) {
            return Err($crate::AppError::Forbidden);
        }
    };
}

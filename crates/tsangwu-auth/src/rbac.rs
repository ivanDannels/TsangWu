use tsangwu_common::AppError;
use tsangwu_db::DbConn;
use tsangwu_db::repositories::role_repo::RoleRepo;
use tsangwu_db::repositories::user_repo::UserRepo;

use crate::jwt::Claims;

/// RBAC 权限校验
pub async fn require_permission(
    claims: &Claims,
    resource: &str,
    action: &str,
    db: &DbConn,
) -> Result<(), AppError> {
    // 平台管理员拥有所有权限
    if claims.roles.contains(&"platform_admin".to_string()) {
        return Ok(());
    }

    let uid: uuid::Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;
    let user = UserRepo::find_by_uid(db, uid)
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("数据库查询失败")))?
        .ok_or(AppError::Unauthorized)?;

    let has = RoleRepo::has_permission(db, user.id, resource, action)
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("权限查询失败")))?;

    if !has {
        tracing::warn!(
            user = %claims.sub,
            resource = resource,
            action = action,
            "权限校验未通过"
        );
        return Err(AppError::Forbidden);
    }

    tracing::debug!(
        user = %claims.sub,
        resource = resource,
        action = action,
        "权限校验通过"
    );

    Ok(())
}

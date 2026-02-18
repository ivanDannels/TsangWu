use axum::extract::{Extension, Path};
use axum::{routing::{get, post, put}, Json};
use sea_orm::*;
use tsangwu_auth::Claims;
use tsangwu_common::{ApiResponse, AppError, PageParams};
use tsangwu_db::DbConn;

async fn list_notifications(
    Extension(db): Extension<DbConn>,
    Extension(claims): Extension<Claims>,
    axum::extract::Query(params): axum::extract::Query<PageParams>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let uid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::Unauthorized)?;
    let (items, total) = tsangwu_db::repositories::notification_repo::NotificationRepo::list_by_user(&db, user.id, &params).await
        .map_err(|e| AppError::Internal(e))?;
    let unread = tsangwu_db::repositories::notification_repo::NotificationRepo::unread_count(&db, user.id).await
        .map_err(|e| AppError::Internal(e))?;
    let notifs: Vec<serde_json::Value> = items.into_iter().map(|n| serde_json::json!({
        "id": n.id,
        "title": n.title,
        "content": n.content,
        "read": n.read,
        "created_at": n.created_at.to_string(),
    })).collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "notifications": notifs,
        "total": total,
        "unread_count": unread,
    }))))
}

async fn mark_read(
    Extension(db): Extension<DbConn>,
    Path(notif_id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    tsangwu_db::repositories::notification_repo::NotificationRepo::mark_read(&db, notif_id).await
        .map_err(|e| AppError::Internal(e))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"ok": true}))))
}

async fn mark_all_read(
    Extension(db): Extension<DbConn>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let uid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::Unauthorized)?;
    tsangwu_db::repositories::notification_repo::NotificationRepo::mark_all_read(&db, user.id).await
        .map_err(|e| AppError::Internal(e))?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"ok": true}))))
}

async fn send_notification(
    Extension(db): Extension<DbConn>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let user_id = body["user_id"].as_i64().ok_or(AppError::Validation("缺少 user_id".into()))?;
    let title = body["title"].as_str().unwrap_or("通知");
    let content = body["content"].as_str().unwrap_or("");
    let now = chrono::Utc::now().fixed_offset();

    let model = tsangwu_db::entities::notification::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        user_id: Set(user_id),
        title: Set(title.to_string()),
        content: Set(content.to_string()),
        read: Set(false),
        created_at: Set(now),
    };
    let notif = tsangwu_db::repositories::notification_repo::NotificationRepo::create(&db, model).await
        .map_err(|e| AppError::Internal(e))?;
    tracing::info!(notif_id = notif.id, "通知已发送");
    Ok(Json(ApiResponse::ok(serde_json::json!({"sent": true, "id": notif.id}))))
}

pub fn routes<S: Clone + Send + Sync + 'static>() -> axum::Router<S> {
    axum::Router::new()
        .route("/v1/notifications", get(list_notifications))
        .route("/v1/notifications", post(send_notification))
        .route("/v1/notifications/{notif_id}/read", put(mark_read))
        .route("/v1/notifications/read-all", put(mark_all_read))
}

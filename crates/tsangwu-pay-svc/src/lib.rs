pub mod gateway;

use axum::extract::{Extension, Path};
use chrono::Datelike;
use tsangwu_auth::Claims;
use tsangwu_common::{ApiResponse, AppError};
use axum::Json;
use tsangwu_db::DbConn;

async fn get_subscription(
    Extension(db): Extension<DbConn>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let uid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::Unauthorized)?;
    let sub = tsangwu_db::repositories::subscription_repo::SubscriptionRepo::find_active_by_user(&db, user.id).await
        .map_err(|e| AppError::Internal(e))?;
    match sub {
        Some(s) => Ok(Json(ApiResponse::ok(serde_json::json!({
            "plan": s.plan_code,
            "status": s.status,
            "expires_at": s.expires_at.to_string(),
            "auto_renew": s.auto_renew,
        })))),
        None => Ok(Json(ApiResponse::ok(serde_json::json!({
            "plan": "free",
            "status": "active",
            "expires_at": null,
        })))),
    }
}

async fn create_order(
    Extension(_db): Extension<DbConn>,
    Extension(_claims): Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let plan = body["plan"].as_str().unwrap_or("personal");
    let gw = gateway::MockPaymentGateway;
    let pay_url = gw.create_payment(plan, 9900).await;
    let order_id = uuid::Uuid::new_v4().to_string();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "order_id": order_id,
        "pay_url": pay_url,
        "plan": plan,
    }))))
}

async fn payment_callback(
    Extension(_db): Extension<DbConn>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let order_id = body["order_id"].as_str().unwrap_or("unknown");
    let status = body["status"].as_str().unwrap_or("success");
    tracing::info!(order_id = %order_id, status = %status, "支付回调");
    // 模拟：验证签名 → 更新订单状态 → 激活订阅
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "received": true,
        "order_id": order_id,
    }))))
}

async fn get_usage(
    Extension(db): Extension<DbConn>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let uid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = tsangwu_db::repositories::user_repo::UserRepo::find_by_uid(&db, uid).await
        .map_err(|e| AppError::Internal(e))?.ok_or(AppError::Unauthorized)?;

    let now = chrono::Utc::now().fixed_offset();
    let naive_now = now.naive_local();
    let first_of_month = chrono::NaiveDate::from_ymd_opt(naive_now.year(), naive_now.month(), 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let from = chrono::DateTime::<chrono::FixedOffset>::from_naive_utc_and_offset(first_of_month, *now.offset());
    let total = tsangwu_db::repositories::usage_repo::UsageRepo::sum_by_user_period(&db, user.id, from, now).await
        .map_err(|e| AppError::Internal(e))?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "total_generations": total,
        "period": "current_month",
    }))))
}

async fn get_order(
    Extension(_db): Extension<DbConn>,
    Path(order_id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "order_id": order_id,
        "status": "pending",
    }))))
}

pub fn routes<S: Clone + Send + Sync + 'static>() -> axum::Router<S> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/v1/subscription", get(get_subscription))
        .route("/v1/orders", post(create_order))
        .route("/v1/orders/{order_id}", get(get_order))
        .route("/v1/payments/callback", post(payment_callback))
        .route("/v1/usage", get(get_usage))
}

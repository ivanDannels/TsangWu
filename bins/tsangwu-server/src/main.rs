use anyhow::Result;
use axum::{middleware, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tsangwu_app::AppState;
use tsangwu_config::TsangwuConfig;
use tsangwu_gateway::{init_metrics, InMemoryRateLimiter, RateLimitConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let profile = std::env::var("TSANGWU_PROFILE").unwrap_or_else(|_| "standalone".into());
    let config = TsangwuConfig::load(&profile)?;

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(&config.telemetry.log_level)
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(mode = ?config.deploy_mode, "苍梧服务启动");

    // 初始化 Prometheus 指标
    init_metrics();
    tracing::info!("Prometheus 指标已初始化");

    // 初始化 AppState
    let db = tsangwu_db::DbConn::from_config(&config.database).await?;
    let jwt = Arc::new(tsangwu_auth::JwtManager::new(
        &std::env::var("TSANGWU_JWT_SECRET").unwrap_or_else(|_| "tsangwu-dev-secret-key-2024".into()),
    ));
    let state = Arc::new(AppState::new(db, jwt.clone()));

    // 初始化限流器
    let rate_limit_config = RateLimitConfig {
        requests_per_minute: 100,
        requests_per_hour: 2000,
    };
    let rate_limiter = Arc::new(InMemoryRateLimiter::new(rate_limit_config));
    let rate_limit_layer = middleware::from_fn_with_state(rate_limiter, tsangwu_gateway::rate_limit_middleware);

    // 认证中间件
    let auth_layer = middleware::from_fn_with_state(jwt, tsangwu_auth::auth_middleware);

    // 需要认证的路由
    let protected_routes = Router::new()
        .merge(tsangwu_agent_svc::routes())
        .merge(tsangwu_gen_svc::routes())
        .merge(tsangwu_project_svc::routes())
        .merge(tsangwu_asset_svc::routes())
        .merge(tsangwu_dist_svc::routes())
        .merge(tsangwu_pay_svc::routes())
        .merge(tsangwu_notif_svc::routes())
        .layer(auth_layer.clone());

    // 公开路由（不需要认证）
    let public_routes = Router::new()
        .merge(tsangwu_user_svc::routes())
        .merge(tsangwu_auth_svc::routes())
        .merge(tsangwu_culture_svc::routes())
        .merge(tsangwu_audit_svc::routes())
        .merge(tsangwu_copyright_svc::routes())
        .merge(tsangwu_template_svc::routes());

    // 组装路由（应用限流）
    let api = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .layer(rate_limit_layer)
        .with_state(state.clone());

    let app = Router::new()
        .nest("/api", api)
        .merge(tsangwu_gateway::health_routes());

    let app = tsangwu_gateway::gateway_layers(app);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!(addr = %addr, "HTTP 服务监听");
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

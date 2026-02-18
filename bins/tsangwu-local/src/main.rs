use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tsangwu_app::AppState;
use tsangwu_config::TsangwuConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let config = TsangwuConfig::load_local()?;

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(&config.telemetry.log_level)
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("苍梧本地模式启动");

    // 确保数据目录存在
    let data_dir = std::env::var("TSANGWU_DATA_DIR").unwrap_or_else(|_| "./data".into());
    tokio::fs::create_dir_all(format!("{}/storage", data_dir)).await?;

    // 初始化 AppState
    let db = tsangwu_db::DbConn::from_config(&config.database).await?;
    let jwt = Arc::new(tsangwu_auth::JwtManager::new(
        &std::env::var("TSANGWU_JWT_SECRET").unwrap_or_else(|_| "tsangwu-local-dev-key".into()),
    ));
    let state = Arc::new(AppState::new(db, jwt));

    // 组装 API 路由
    let api = Router::new()
        .merge(tsangwu_user_svc::routes())
        .merge(tsangwu_auth_svc::routes())
        .merge(tsangwu_project_svc::routes())
        .merge(tsangwu_gen_svc::routes())
        .merge(tsangwu_asset_svc::routes())
        .merge(tsangwu_template_svc::routes())
        .merge(tsangwu_culture_svc::routes())
        .merge(tsangwu_agent_svc::routes())
        .merge(tsangwu_audit_svc::routes())
        .merge(tsangwu_copyright_svc::routes())
        .merge(tsangwu_pay_svc::routes())
        .merge(tsangwu_notif_svc::routes())
        .with_state(state.clone());

    // 主应用路由
    let app = Router::new()
        .nest("/api", api)
        .route("/healthz", axum::routing::get(|| async { "ok" }))
        .route("/readyz", axum::routing::get(|| async { "ok" }))
        // 静态文件服务
        .nest_service("/static", ServeDir::new("web/static"))
        // 主页
        .route("/", axum::routing::get(|| async { axum::response::Html::from(include_str!("../../../web/static/index.html")) }))
        // 其他路径回退到 index.html (SPA 支持)
        .fallback_service(ServeFile::new("web/static/index.html"));

    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!(addr = %addr, "本地模式 HTTP 监听");
    tracing::info!("访问 http://{} 查看界面", addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

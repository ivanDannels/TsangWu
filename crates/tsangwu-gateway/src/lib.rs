use axum::Router;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};

pub mod metrics;
pub mod rate_limit;

pub use metrics::{
    init_metrics, http_metrics_middleware,
    agent_metrics, culture_metrics, rate_limit_metrics,
};
pub use rate_limit::{
    InMemoryRateLimiter, RateLimitConfig, CacheRateLimiter,
    rate_limit_middleware, cache_rate_limit_middleware,
};

/// 构建 API 网关中间件栈
pub fn gateway_layers<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    router
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(100 * 1024 * 1024)) // 100MB
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

/// 健康检查路由
pub fn health_routes() -> Router {
    use axum::routing::get;
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ok" }))
        .route("/metrics", get(get_metrics_output))
}

/// 获取 Prometheus 指标输出
async fn get_metrics_output() -> String {
    metrics::get_metrics_output()
}

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// 请求追踪 ID 的 Header 名称
pub const TRACE_ID_HEADER: &str = "x-trace-id";

/// 追踪中间件：为每个请求生成或传递 trace_id
pub async fn trace_middleware(mut req: Request, next: Next) -> Response {
    // 尝试从请求头获取 trace_id，如果没有则生成新的
    let trace_id = req
        .headers()
        .get(TRACE_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 将 trace_id 添加到请求扩展中，供后续处理器使用
    req.extensions_mut().insert(TraceId(trace_id.clone()));

    // 记录请求开始
    tracing::info!(
        trace_id = %trace_id,
        method = %req.method(),
        uri = %req.uri(),
        "请求开始"
    );

    // 执行请求
    let mut response = next.run(req).await;

    // 将 trace_id 添加到响应头
    if let Ok(header_value) = HeaderValue::from_str(&trace_id) {
        response.headers_mut().insert(TRACE_ID_HEADER, header_value);
    }

    response
}

/// 追踪 ID 包装类型
#[derive(Clone, Debug)]
pub struct TraceId(pub String);

impl TraceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

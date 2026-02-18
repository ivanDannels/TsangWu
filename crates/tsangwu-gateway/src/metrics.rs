use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::sync::OnceLock;
use std::time::Instant;

static RECORDER: OnceLock<String> = OnceLock::new();

/// 初始化 Prometheus 指标导出器
pub fn init_metrics() {
    let builder = PrometheusBuilder::new();
    let handle = builder.install_recorder().expect("Failed to install Prometheus recorder");
    let _ = RECORDER.set(handle.render());
}

/// 获取 Prometheus 指标输出
pub fn get_metrics_output() -> String {
    PrometheusBuilder::new()
        .build_recorder()
        .handle()
        .render()
}

/// HTTP 请求指标中间件
pub async fn http_metrics_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_string()
    } else {
        req.uri().path().to_string()
    };

    // 增加请求计数
    counter!(
        "http_requests_total",
        "method" => method.to_string(),
        "path" => path.clone()
    ).increment(1);

    // 增加活跃请求计数
    gauge!(
        "http_requests_in_flight",
        "method" => method.to_string(),
        "path" => path.clone()
    ).increment(1.0);

    let response = next.run(req).await;

    // 减少活跃请求计数
    gauge!(
        "http_requests_in_flight",
        "method" => method.to_string(),
        "path" => path.clone()
    ).decrement(1.0);

    let elapsed = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // 记录请求延迟
    histogram!(
        "http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path,
        "status" => status
    ).record(elapsed);

    response
}

/// Agent 任务指标
pub mod agent_metrics {
    use metrics::{counter, gauge, histogram};

    /// 记录任务创建
    pub fn task_created(agent_type: &str, task_type: &str) {
        counter!(
            "agent_tasks_created_total",
            "agent_type" => agent_type.to_string(),
            "task_type" => task_type.to_string()
        ).increment(1);
    }

    /// 记录任务完成
    pub fn task_completed(agent_type: &str, task_type: &str, status: &str, duration_secs: f64) {
        counter!(
            "agent_tasks_completed_total",
            "agent_type" => agent_type.to_string(),
            "task_type" => task_type.to_string(),
            "status" => status.to_string()
        ).increment(1);

        histogram!(
            "agent_task_duration_seconds",
            "agent_type" => agent_type.to_string(),
            "task_type" => task_type.to_string()
        ).record(duration_secs);
    }

    /// 更新活跃任务数
    pub fn active_tasks(agent_type: &str, count: f64) {
        gauge!(
            "agent_active_tasks",
            "agent_type" => agent_type.to_string()
        ).set(count);
    }

    /// 记录 Agent 创建
    pub fn agent_created(agent_type: &str) {
        counter!(
            "agents_created_total",
            "agent_type" => agent_type.to_string()
        ).increment(1);
    }

    /// 更新 Agent 状态
    pub fn agent_status(agent_type: &str, status: &str, count: f64) {
        gauge!(
            "agents_by_status",
            "agent_type" => agent_type.to_string(),
            "status" => status.to_string()
        ).set(count);
    }
}

/// 文化引擎指标
pub mod culture_metrics {
    use metrics::{counter, histogram};

    /// 记录验证请求
    pub fn validation_requested(dynasty: &str) {
        counter!(
            "culture_validations_total",
            "dynasty" => dynasty.to_string()
        ).increment(1);
    }

    /// 记录验证结果
    pub fn validation_completed(dynasty: &str, valid: bool, score: f32, duration_secs: f64) {
        counter!(
            "culture_validations_completed_total",
            "dynasty" => dynasty.to_string(),
            "valid" => valid.to_string()
        ).increment(1);

        histogram!(
            "culture_validation_score",
            "dynasty" => dynasty.to_string()
        ).record(score as f64);

        histogram!(
            "culture_validation_duration_seconds",
            "dynasty" => dynasty.to_string()
        ).record(duration_secs);
    }

    /// 记录推荐请求
    pub fn recommendation_requested(dynasty: &str, element_type: &str) {
        counter!(
            "culture_recommendations_total",
            "dynasty" => dynasty.to_string(),
            "element_type" => element_type.to_string()
        ).increment(1);
    }

    /// 记录缓存命中
    pub fn cache_hit(cache_type: &str) {
        counter!(
            "culture_cache_hits_total",
            "cache_type" => cache_type.to_string()
        ).increment(1);
    }

    /// 记录缓存未命中
    pub fn cache_miss(cache_type: &str) {
        counter!(
            "culture_cache_misses_total",
            "cache_type" => cache_type.to_string()
        ).increment(1);
    }
}

/// 限流指标
pub mod rate_limit_metrics {
    use metrics::counter;

    /// 记录限流触发
    pub fn rate_limited(client_ip: &str, limit_type: &str) {
        counter!(
            "rate_limit_triggered_total",
            "client_ip" => client_ip.to_string(),
            "limit_type" => limit_type.to_string()
        ).increment(1);
    }

    /// 记录限流检查
    pub fn rate_limit_check(client_ip: &str, allowed: bool) {
        counter!(
            "rate_limit_checks_total",
            "client_ip" => client_ip.to_string(),
            "allowed" => allowed.to_string()
        ).increment(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        // 初始化指标应该不会 panic
        init_metrics();
    }
}

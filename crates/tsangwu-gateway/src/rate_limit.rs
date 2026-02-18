use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tsangwu_cache::CacheBackend;

/// 限流配置
#[derive(Clone)]
pub struct RateLimitConfig {
    /// 每个IP每分钟最大请求数
    pub requests_per_minute: u64,
    /// 每个IP每小时最大请求数
    pub requests_per_hour: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
        }
    }
}

/// 内存限流器（用于单机部署）
#[derive(Clone)]
pub struct InMemoryRateLimiter {
    config: RateLimitConfig,
    minute_counts: Arc<RwLock<Vec<(String, u32, Instant)>>>,
    hour_counts: Arc<RwLock<Vec<(String, u32, Instant)>>>,
}

impl InMemoryRateLimiter {
    /// 创建新的内存限流器
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            minute_counts: Arc::new(RwLock::new(Vec::new())),
            hour_counts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 检查是否允许请求
    pub async fn check(&self, key: &str) -> Result<(), RateLimitError> {
        let now = Instant::now();

        // 检查分钟限制
        {
            let mut counts = self.minute_counts.write().await;
            // 清理过期记录
            counts.retain(|(_, _, ts)| now.duration_since(*ts) < Duration::from_secs(60));
            
            let count = counts.iter()
                .filter(|(k, _, _)| k == key)
                .count() as u64;
            
            if count >= self.config.requests_per_minute {
                return Err(RateLimitError::MinuteLimitExceeded);
            }
            
            counts.push((key.to_string(), 1, now));
        }

        // 检查小时限制
        {
            let mut counts = self.hour_counts.write().await;
            // 清理过期记录
            counts.retain(|(_, _, ts)| now.duration_since(*ts) < Duration::from_secs(3600));
            
            let count = counts.iter()
                .filter(|(k, _, _)| k == key)
                .count() as u64;
            
            if count >= self.config.requests_per_hour {
                return Err(RateLimitError::HourLimitExceeded);
            }
            
            counts.push((key.to_string(), 1, now));
        }

        Ok(())
    }
}

/// 限流错误类型
#[derive(Debug)]
pub enum RateLimitError {
    MinuteLimitExceeded,
    HourLimitExceeded,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        let (message, retry_after) = match self {
            RateLimitError::MinuteLimitExceeded => ("请求过于频繁，请稍后再试", 60),
            RateLimitError::HourLimitExceeded => ("已达到小时请求限制", 3600),
        };
        
        let body = Json(serde_json::json!({
            "error": message,
            "retry_after_seconds": retry_after
        }));

        (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("X-RateLimit-Reset", retry_after.to_string()),
                ("Retry-After", retry_after.to_string()),
            ],
            body,
        )
            .into_response()
    }
}

/// 基于缓存的分布式限流器
pub struct CacheRateLimiter {
    cache: Arc<dyn CacheBackend>,
    config: RateLimitConfig,
}

impl CacheRateLimiter {
    /// 创建新的缓存限流器
    pub fn new(cache: Arc<dyn CacheBackend>, config: RateLimitConfig) -> Self {
        Self { cache, config }
    }

    /// 检查是否允许请求
    pub async fn check(&self, key: &str) -> Result<(), RateLimitError> {
        let minute_key = format!("ratelimit:{}:minute", key);
        let hour_key = format!("ratelimit:{}:hour", key);

        // 检查分钟限制
        let minute_count = self.cache.incr(&minute_key, 60).await.unwrap_or(1);
        if minute_count > self.config.requests_per_minute {
            return Err(RateLimitError::MinuteLimitExceeded);
        }

        // 检查小时限制
        let hour_count = self.cache.incr(&hour_key, 3600).await.unwrap_or(1);
        if hour_count > self.config.requests_per_hour {
            return Err(RateLimitError::HourLimitExceeded);
        }

        Ok(())
    }
}

/// 从请求中提取客户端标识
fn extract_client_key(req: &Request) -> String {
    // 优先使用 X-Forwarded-For 头
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return format!("ip:{}", first_ip.trim());
            }
        }
    }
    
    // 其次使用 X-Real-IP 头
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip) = real_ip.to_str() {
            return format!("ip:{}", ip);
        }
    }
    
    // 默认使用 unknown
    "ip:unknown".to_string()
}

/// 内存限流中间件
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<InMemoryRateLimiter>>,
    req: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    let key = extract_client_key(&req);
    limiter.check(&key).await?;
    Ok(next.run(req).await)
}

/// 基于缓存的限流中间件
pub async fn cache_rate_limit_middleware(
    State(limiter): State<Arc<CacheRateLimiter>>,
    req: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    let key = extract_client_key(&req);
    limiter.check(&key).await?;
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 60);
        assert_eq!(config.requests_per_hour, 1000);
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_allows_requests() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            requests_per_hour: 100,
        };
        let limiter = InMemoryRateLimiter::new(config);

        // 应该允许前10个请求
        for _ in 0..10 {
            assert!(limiter.check("test_client").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_blocks_excess() {
        let config = RateLimitConfig {
            requests_per_minute: 5,
            requests_per_hour: 100,
        };
        let limiter = InMemoryRateLimiter::new(config);

        // 允许前5个请求
        for _ in 0..5 {
            assert!(limiter.check("test_client_2").await.is_ok());
        }

        // 第6个请求应该被拒绝
        let result = limiter.check("test_client_2").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RateLimitError::MinuteLimitExceeded));
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_different_clients() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            requests_per_hour: 10,
        };
        let limiter = InMemoryRateLimiter::new(config);

        // 不同的客户端应该有独立的计数
        assert!(limiter.check("client_a").await.is_ok());
        assert!(limiter.check("client_a").await.is_ok());
        assert!(limiter.check("client_b").await.is_ok());
        assert!(limiter.check("client_b").await.is_ok());

        // 两个客户端都应该被限制
        assert!(limiter.check("client_a").await.is_err());
        assert!(limiter.check("client_b").await.is_err());
    }
}

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::HashMap;

/// 限流器配置
#[derive(Clone, Debug)]
pub struct RateLimiterConfig {
    /// 时间窗口（秒）
    pub window_secs: u64,
    /// 窗口内最大请求数
    pub max_requests: u32,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            window_secs: 60,
            max_requests: 100,
        }
    }
}

/// 限流器
pub struct RateLimiter {
    config: RateLimiterConfig,
    buckets: Arc<RwLock<HashMap<String, Bucket>>>,
}

struct Bucket {
    count: u32,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 检查是否允许请求
    pub async fn check(&self, key: &str) -> bool {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();

        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            count: 0,
            window_start: now,
        });

        // 检查窗口是否过期
        if now.duration_since(bucket.window_start) > Duration::from_secs(self.config.window_secs) {
            bucket.count = 0;
            bucket.window_start = now;
        }

        // 检查是否超过限制
        if bucket.count >= self.config.max_requests {
            return false;
        }

        bucket.count += 1;
        true
    }

    /// 清理过期的桶
    pub async fn cleanup(&self) {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_secs);

        buckets.retain(|_, bucket| now.duration_since(bucket.window_start) <= window);
    }
}

/// 熔断器状态
#[derive(Clone, Debug, PartialEq)]
pub enum CircuitState {
    /// 关闭状态（正常）
    Closed,
    /// 打开状态（熔断）
    Open,
    /// 半开状态（尝试恢复）
    HalfOpen,
}

/// 熔断器配置
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// 失败阈值
    pub failure_threshold: u32,
    /// 成功阈值（半开状态下）
    pub success_threshold: u32,
    /// 超时时间（秒）
    pub timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout_secs: 60,
        }
    }
}

/// 熔断器
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    success_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    /// 检查是否允许请求
    pub async fn allow_request(&self) -> bool {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // 检查是否可以进入半开状态
                let last_failure = self.last_failure_time.read().await;
                if let Some(time) = *last_failure {
                    if Instant::now().duration_since(time) > Duration::from_secs(self.config.timeout_secs) {
                        *state = CircuitState::HalfOpen;
                        *self.success_count.write().await = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// 记录成功
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::Closed => {
                *self.failure_count.write().await = 0;
            }
            CircuitState::HalfOpen => {
                let mut success_count = self.success_count.write().await;
                *success_count += 1;

                if *success_count >= self.config.success_threshold {
                    *state = CircuitState::Closed;
                    *self.failure_count.write().await = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    /// 记录失败
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;

        *failure_count += 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        if *failure_count >= self.config.failure_threshold {
            *state = CircuitState::Open;
        }

        if *state == CircuitState::HalfOpen {
            *state = CircuitState::Open;
        }
    }

    /// 获取当前状态
    pub async fn state(&self) -> CircuitState {
        self.state.read().await.clone()
    }
}

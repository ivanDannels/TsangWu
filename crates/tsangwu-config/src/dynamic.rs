use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 敏感配置项（从环境变量读取）
#[derive(Clone, Debug)]
pub struct SecretConfig {
    pub jwt_secret: String,
    pub db_password: Option<String>,
    pub s3_secret_key: Option<String>,
    pub api_keys: Vec<String>,
}

impl SecretConfig {
    /// 从环境变量加载敏感配置
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            jwt_secret: std::env::var("TSANGWU_JWT_SECRET")
                .unwrap_or_else(|_| "default-dev-secret-change-in-production".to_string()),
            db_password: std::env::var("TSANGWU_DB_PASSWORD").ok(),
            s3_secret_key: std::env::var("TSANGWU_S3_SECRET_KEY").ok(),
            api_keys: std::env::var("TSANGWU_API_KEYS")
                .ok()
                .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
                .unwrap_or_default(),
        })
    }
}

/// 可热更新的配置项
#[derive(Clone, Debug, Deserialize)]
pub struct DynamicConfig {
    /// 限流配置
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    /// 熔断配置
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
    /// 功能开关
    #[serde(default)]
    pub feature_flags: FeatureFlags,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_rate_limit_enabled")]
    pub enabled: bool,
    #[serde(default = "default_rate_limit_window")]
    pub window_secs: u64,
    #[serde(default = "default_rate_limit_max")]
    pub max_requests: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_secs: 60,
            max_requests: 100,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CircuitBreakerConfig {
    #[serde(default = "default_circuit_breaker_enabled")]
    pub enabled: bool,
    #[serde(default = "default_circuit_breaker_threshold")]
    pub failure_threshold: u32,
    #[serde(default = "default_circuit_breaker_timeout")]
    pub timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 5,
            timeout_secs: 60,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub enable_cache: bool,
    #[serde(default = "default_true")]
    pub enable_tracing: bool,
    #[serde(default = "default_false")]
    pub enable_experimental: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_cache: true,
            enable_tracing: true,
            enable_experimental: false,
        }
    }
}

fn default_rate_limit_enabled() -> bool { true }
fn default_rate_limit_window() -> u64 { 60 }
fn default_rate_limit_max() -> u32 { 100 }
fn default_circuit_breaker_enabled() -> bool { true }
fn default_circuit_breaker_threshold() -> u32 { 5 }
fn default_circuit_breaker_timeout() -> u64 { 60 }
fn default_true() -> bool { true }
fn default_false() -> bool { false }

impl Default for DynamicConfig {
    fn default() -> Self {
        Self {
            rate_limit: RateLimitConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            feature_flags: FeatureFlags::default(),
        }
    }
}

/// 配置管理器（支持热更新）
pub struct ConfigManager {
    dynamic: Arc<RwLock<DynamicConfig>>,
}

impl ConfigManager {
    pub fn new(config: DynamicConfig) -> Self {
        Self {
            dynamic: Arc::new(RwLock::new(config)),
        }
    }

    /// 获取当前动态配置
    pub async fn get(&self) -> DynamicConfig {
        let config = self.dynamic.read().await;
        config.clone()
    }

    /// 热更新配置
    pub async fn update(&self, new_config: DynamicConfig) {
        let mut config = self.dynamic.write().await;
        *config = new_config;
        tracing::info!("配置已热更新");
    }

    /// 从文件重新加载配置
    pub async fn reload_from_file(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let content = tokio::fs::read_to_string(path).await?;
        let new_config: DynamicConfig = serde_json::from_str(&content)?;
        self.update(new_config).await;
        Ok(())
    }
}

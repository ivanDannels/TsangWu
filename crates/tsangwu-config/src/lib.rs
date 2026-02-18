pub mod dynamic;

use serde::Deserialize;

pub use dynamic::{ConfigManager, DynamicConfig, SecretConfig};

/// 部署模式
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeployMode {
    Local,
    Standalone,
    Distributed,
}

/// 顶层配置
#[derive(Debug, Clone, Deserialize)]
pub struct TsangwuConfig {
    pub deploy_mode: DeployMode,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub storage: StorageConfig,
    pub message_queue: MqConfig,
    pub search: SearchConfig,
    pub workflow: WorkflowConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DatabaseConfig {
    Postgres {
        url: String,
        #[serde(default = "default_max_conn")]
        max_connections: u32,
    },
    Sqlite {
        path: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CacheConfig {
    Redis { url: String },
    Memory {
        #[serde(default = "default_cache_capacity")]
        max_capacity: u64,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StorageConfig {
    #[serde(rename = "local_fs")]
    LocalFs { root: String },
    S3 {
        endpoint: String,
        bucket: String,
        access_key: String,
        secret_key: String,
        #[serde(default)]
        base_url: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MqConfig {
    Kafka {
        brokers: Vec<String>,
    },
    Memory {
        #[serde(default = "default_channel_size")]
        channel_size: usize,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SearchConfig {
    Elasticsearch { url: String },
    #[serde(rename = "postgres")]
    PostgresFts,
    #[serde(rename = "sqlite")]
    SqliteFts,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WorkflowConfig {
    Embedded {
        persistence: String, // "sqlite" or "postgres"
    },
    Distributed {
        persistence: String,
        event_bus: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            log_level: "info".into(),
            otlp_endpoint: None,
            service_name: "tsangwu".into(),
        }
    }
}

// --- 默认值函数 ---

fn default_host() -> String {
    "0.0.0.0".into()
}
fn default_port() -> u16 {
    8080
}
fn default_grpc_port() -> u16 {
    9090
}
fn default_max_conn() -> u32 {
    20
}
fn default_cache_capacity() -> u64 {
    10000
}
fn default_channel_size() -> usize {
    1000
}
fn default_log_level() -> String {
    "info".into()
}
fn default_service_name() -> String {
    "tsangwu".into()
}

// --- 配置加载 ---

impl TsangwuConfig {
    /// 从配置文件 + 环境变量加载配置
    pub fn load(config_name: &str) -> anyhow::Result<Self> {
        let config_dir = std::env::var("TSANGWU_CONFIG_DIR").unwrap_or_else(|_| "config".into());
        let cfg = config::Config::builder()
            .add_source(config::File::with_name(&format!("{}/{}", config_dir, config_name)))
            .add_source(config::Environment::with_prefix("TSANGWU").separator("__"))
            .build()?;
        let config: TsangwuConfig = cfg.try_deserialize()?;
        Ok(config)
    }

    /// 加载本地模式默认配置
    pub fn load_local() -> anyhow::Result<Self> {
        let data_dir = std::env::var("TSANGWU_DATA_DIR").unwrap_or_else(|_| "./data".into());
        Ok(Self {
            deploy_mode: DeployMode::Local,
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: std::env::var("TSANGWU_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080),
                grpc_port: 0,
            },
            database: DatabaseConfig::Sqlite {
                path: format!("{}/tsangwu.db", data_dir),
            },
            cache: CacheConfig::Memory { max_capacity: 10000 },
            storage: StorageConfig::LocalFs {
                root: format!("{}/storage", data_dir),
            },
            message_queue: MqConfig::Memory { channel_size: 1000 },
            search: SearchConfig::SqliteFts,
            workflow: WorkflowConfig::Embedded {
                persistence: "sqlite".into(),
            },
            telemetry: TelemetryConfig::default(),
        })
    }
}

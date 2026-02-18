use sea_orm::{Database, DatabaseConnection, DbErr};
use std::sync::Arc;
use tsangwu_config::{CacheConfig, DatabaseConfig, MqConfig, StorageConfig};

/// 测试数据库连接
pub struct TestDb {
    pub conn: DatabaseConnection,
    db_name: String,
}

impl TestDb {
    /// 创建测试数据库（使用内存 SQLite）
    pub async fn new() -> Result<Self, DbErr> {
        let db_name = format!("test_{}", uuid::Uuid::new_v4());
        let conn = Database::connect(format!("sqlite::memory:")).await?;

        Ok(Self { conn, db_name })
    }

    /// 获取数据库连接
    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// 运行迁移（需要手动实现）
    pub async fn migrate(&self) -> Result<(), DbErr> {
        // 这里可以添加测试迁移逻辑
        Ok(())
    }

    /// 清理数据库
    pub async fn cleanup(&self) -> Result<(), DbErr> {
        // SQLite 内存数据库会自动清理
        Ok(())
    }
}

/// 测试应用状态
pub struct TestAppState {
    pub db: crate::DbConn,
    pub jwt: Arc<tsangwu_auth::JwtManager>,
}

impl TestAppState {
    /// 创建测试应用状态
    pub async fn new() -> anyhow::Result<Self> {
        let test_db = TestDb::new().await?;
        let db = crate::DbConn {
            conn: test_db.conn,
        };

        let jwt = Arc::new(tsangwu_auth::JwtManager::new("test-secret-key"));

        Ok(Self { db, jwt })
    }
}

/// 测试配置构建器
pub struct TestConfigBuilder {
    database: Option<DatabaseConfig>,
    cache: Option<CacheConfig>,
    storage: Option<StorageConfig>,
    mq: Option<MqConfig>,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            database: None,
            cache: None,
            storage: None,
            mq: None,
        }
    }

    pub fn with_database(mut self, config: DatabaseConfig) -> Self {
        self.database = Some(config);
        self
    }

    pub fn with_cache(mut self, config: CacheConfig) -> Self {
        self.cache = Some(config);
        self
    }

    pub fn with_storage(mut self, config: StorageConfig) -> Self {
        self.storage = Some(config);
        self
    }

    pub fn with_mq(mut self, config: MqConfig) -> Self {
        self.mq = Some(config);
        self
    }

    /// 构建测试配置（使用默认值）
    pub fn build(self) -> TestConfig {
        TestConfig {
            database: self.database.unwrap_or(DatabaseConfig::Sqlite {
                path: ":memory:".to_string(),
            }),
            cache: self.cache.unwrap_or(CacheConfig::Memory {
                max_capacity: 1000,
            }),
            storage: self.storage.unwrap_or(StorageConfig::LocalFs {
                root: "/tmp/tsangwu-test".to_string(),
            }),
            mq: self.mq.unwrap_or(MqConfig::Memory {
                channel_size: 100,
            }),
        }
    }
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TestConfig {
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub storage: StorageConfig,
    pub mq: MqConfig,
}

/// 测试辅助宏
#[macro_export]
macro_rules! test_db {
    () => {{
        $crate::test_utils::TestDb::new().await.expect("创建测试数据库失败")
    }};
}

#[macro_export]
macro_rules! test_app_state {
    () => {{
        $crate::test_utils::TestAppState::new().await.expect("创建测试应用状态失败")
    }};
}

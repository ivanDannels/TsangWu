use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use tsangwu_config::DatabaseConfig;

/// SeaORM 数据库连接封装
#[derive(Clone)]
pub struct DbConn {
    pub conn: DatabaseConnection,
    #[cfg(feature = "postgres")]
    pg_pool: Option<sqlx::PgPool>,
}

impl DbConn {
    pub async fn from_config(config: &DatabaseConfig) -> anyhow::Result<Self> {
        let url = match config {
            #[cfg(feature = "postgres")]
            DatabaseConfig::Postgres { url, .. } => url.clone(),
            #[cfg(feature = "sqlite")]
            DatabaseConfig::Sqlite { path } => {
                if let Some(parent) = std::path::Path::new(path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                format!("sqlite:{}?mode=rwc", path)
            }
            #[allow(unreachable_patterns)]
            _ => anyhow::bail!("当前编译未启用对应数据库 feature"),
        };
        let conn = sea_orm::Database::connect(&url).await?;
        tracing::info!("SeaORM 数据库连接已建立");
        
        #[cfg(feature = "postgres")]
        {
            let pg_pool = if matches!(config, DatabaseConfig::Postgres { .. }) {
                let pg_url = match config {
                    DatabaseConfig::Postgres { url, max_connections } => {
                        Some(sqlx::postgres::PgPoolOptions::new()
                            .max_connections(*max_connections)
                            .connect(url)
                            .await?)
                    }
                    _ => None,
                };
                pg_url
            } else {
                None
            };
            Ok(Self { conn, pg_pool })
        }
        
        #[cfg(not(feature = "postgres"))]
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// 获取 PostgreSQL 连接池（用于 sqlx 直接查询）
    #[cfg(feature = "postgres")]
    pub fn pg_pool(&self) -> sqlx::PgPool {
        self.pg_pool.clone().expect("PostgreSQL 连接池未初始化")
    }

    /// 开启事务
    pub async fn begin(&self) -> anyhow::Result<DatabaseTransaction> {
        Ok(self.conn.begin().await?)
    }

    /// 执行事务闭包
    pub async fn transaction<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<T>> + Send + 'c>> + Send,
        T: Send,
    {
        let txn = self.begin().await?;
        match f(&txn).await {
            Ok(result) => {
                txn.commit().await?;
                Ok(result)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }
}

/// 数据库连接池抽象
#[derive(Clone)]
pub enum DatabasePool {
    #[cfg(feature = "postgres")]
    Postgres(sqlx::PgPool),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::SqlitePool),
}

impl DatabasePool {
    /// 根据配置初始化数据库连接池并运行迁移
    pub async fn init(config: &DatabaseConfig) -> anyhow::Result<Self> {
        match config {
            #[cfg(feature = "postgres")]
            DatabaseConfig::Postgres { url, max_connections } => {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(*max_connections)
                    .connect(url)
                    .await?;
                tracing::info!("PostgreSQL 连接池已初始化");
                Ok(Self::Postgres(pool))
            }
            #[cfg(feature = "sqlite")]
            DatabaseConfig::Sqlite { path } => {
                if let Some(parent) = std::path::Path::new(path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let url = format!("sqlite:{}?mode=rwc", path);
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&url)
                    .await?;
                tracing::info!("SQLite 连接池已初始化: {}", path);
                Ok(Self::Sqlite(pool))
            }
            #[allow(unreachable_patterns)]
            _ => anyhow::bail!("当前编译未启用对应数据库 feature"),
        }
    }

    /// 获取 PostgreSQL 连接池引用
    #[cfg(feature = "postgres")]
    pub fn pg_pool(&self) -> &sqlx::PgPool {
        match self {
            Self::Postgres(pool) => pool,
            #[allow(unreachable_patterns)]
            _ => panic!("当前非 PostgreSQL 模式"),
        }
    }

    /// 获取 SQLite 连接池引用
    #[cfg(feature = "sqlite")]
    pub fn sqlite_pool(&self) -> &sqlx::SqlitePool {
        match self {
            Self::Sqlite(pool) => pool,
            #[allow(unreachable_patterns)]
            _ => panic!("当前非 SQLite 模式"),
        }
    }
}

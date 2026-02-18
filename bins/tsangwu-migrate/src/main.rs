use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "tsangwu-migrate", about = "苍梧数据库迁移工具")]
struct Cli {
    /// 配置文件名（不含扩展名），默认 standalone
    #[arg(short, long, default_value = "standalone")]
    profile: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 执行所有待运行的迁移
    Up,
    /// 回滚最近一次迁移
    Down,
    /// 显示迁移状态
    Status,
    /// 创建新的迁移文件
    Create {
        /// 迁移名称
        name: String,
    },
}

const MIGRATIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS _migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;

const MIGRATIONS_TABLE_PG_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS _migrations (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
)
"#;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = tsangwu_config::TsangwuConfig::load(&cli.profile)?;
    tracing::info!(mode = ?config.deploy_mode, "加载配置完成");

    let pool = tsangwu_db::DatabasePool::init(&config.database).await?;

    match cli.command {
        Commands::Up => run_up(&pool, &config.database).await?,
        Commands::Down => run_down(&pool, &config.database).await?,
        Commands::Status => run_status(&pool, &config.database).await?,
        Commands::Create { name } => run_create(&name, &config.database)?,
    }

    Ok(())
}

fn migrations_dir(db_config: &tsangwu_config::DatabaseConfig) -> String {
    match db_config {
        #[cfg(feature = "postgres")]
        tsangwu_config::DatabaseConfig::Postgres { .. } => "migrations/postgres".into(),
        #[cfg(feature = "sqlite")]
        tsangwu_config::DatabaseConfig::Sqlite { .. } => "migrations/sqlite".into(),
        #[allow(unreachable_patterns)]
        _ => "migrations".into(),
    }
}

async fn ensure_migrations_table(pool: &tsangwu_db::DatabasePool, db_config: &tsangwu_config::DatabaseConfig) -> Result<()> {
    match db_config {
        #[cfg(feature = "postgres")]
        tsangwu_config::DatabaseConfig::Postgres { .. } => {
            sqlx::query(MIGRATIONS_TABLE_PG_SQL).execute(pool.pg_pool()).await?;
        }
        #[cfg(feature = "sqlite")]
        tsangwu_config::DatabaseConfig::Sqlite { .. } => {
            sqlx::query(MIGRATIONS_TABLE_SQL).execute(pool.sqlite_pool()).await?;
        }
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("不支持的数据库类型"),
    }
    Ok(())
}

fn read_migration_files(dir: &str) -> Result<Vec<(String, String)>> {
    let path = std::path::Path::new(dir);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
        return Ok(vec![]);
    }
    let mut files: Vec<(String, String)> = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "sql").unwrap_or(false))
        .filter(|e| !e.file_name().to_string_lossy().contains(".down."))
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let content = std::fs::read_to_string(e.path()).unwrap_or_default();
            (name, content)
        })
        .collect();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

async fn get_applied_migrations(pool: &tsangwu_db::DatabasePool, db_config: &tsangwu_config::DatabaseConfig) -> Result<Vec<String>> {
    match db_config {
        #[cfg(feature = "postgres")]
        tsangwu_config::DatabaseConfig::Postgres { .. } => {
            let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM _migrations ORDER BY name")
                .fetch_all(pool.pg_pool()).await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
        #[cfg(feature = "sqlite")]
        tsangwu_config::DatabaseConfig::Sqlite { .. } => {
            let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM _migrations ORDER BY name")
                .fetch_all(pool.sqlite_pool()).await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("不支持的数据库类型"),
    }
}

async fn run_up(pool: &tsangwu_db::DatabasePool, db_config: &tsangwu_config::DatabaseConfig) -> Result<()> {
    ensure_migrations_table(pool, db_config).await?;
    let dir = migrations_dir(db_config);
    let files = read_migration_files(&dir)?;
    let applied = get_applied_migrations(pool, db_config).await?;

    let mut count = 0u32;
    for (name, sql) in &files {
        if applied.contains(name) {
            continue;
        }
        tracing::info!(migration = %name, "执行迁移");
        match db_config {
            #[cfg(feature = "postgres")]
            tsangwu_config::DatabaseConfig::Postgres { .. } => {
                sqlx::query(sql).execute(pool.pg_pool()).await?;
                sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
                    .bind(name).execute(pool.pg_pool()).await?;
            }
            #[cfg(feature = "sqlite")]
            tsangwu_config::DatabaseConfig::Sqlite { .. } => {
                sqlx::query(sql).execute(pool.sqlite_pool()).await?;
                sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
                    .bind(name).execute(pool.sqlite_pool()).await?;
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
        count += 1;
    }
    tracing::info!(count = count, "迁移完成");
    Ok(())
}

async fn run_down(pool: &tsangwu_db::DatabasePool, db_config: &tsangwu_config::DatabaseConfig) -> Result<()> {
    ensure_migrations_table(pool, db_config).await?;
    let applied = get_applied_migrations(pool, db_config).await?;
    let last = match applied.last() {
        Some(name) => name.clone(),
        None => {
            tracing::info!("没有可回滚的迁移");
            return Ok(());
        }
    };

    let dir = migrations_dir(db_config);
    let down_name = last.replace(".sql", ".down.sql");
    let down_path = format!("{}/{}", dir, down_name);
    let down_sql = std::fs::read_to_string(&down_path)
        .unwrap_or_else(|_| {
            tracing::warn!(file = %down_path, "未找到回滚文件，跳过SQL执行");
            String::new()
        });

    if !down_sql.is_empty() {
        tracing::info!(migration = %last, "回滚迁移");
        match db_config {
            #[cfg(feature = "postgres")]
            tsangwu_config::DatabaseConfig::Postgres { .. } => {
                sqlx::query(&down_sql).execute(pool.pg_pool()).await?;
            }
            #[cfg(feature = "sqlite")]
            tsangwu_config::DatabaseConfig::Sqlite { .. } => {
                sqlx::query(&down_sql).execute(pool.sqlite_pool()).await?;
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    match db_config {
        #[cfg(feature = "postgres")]
        tsangwu_config::DatabaseConfig::Postgres { .. } => {
            sqlx::query("DELETE FROM _migrations WHERE name = $1")
                .bind(&last).execute(pool.pg_pool()).await?;
        }
        #[cfg(feature = "sqlite")]
        tsangwu_config::DatabaseConfig::Sqlite { .. } => {
            sqlx::query("DELETE FROM _migrations WHERE name = $1")
                .bind(&last).execute(pool.sqlite_pool()).await?;
        }
        #[allow(unreachable_patterns)]
        _ => {}
    }
    tracing::info!("回滚完成");
    Ok(())
}

async fn run_status(pool: &tsangwu_db::DatabasePool, db_config: &tsangwu_config::DatabaseConfig) -> Result<()> {
    ensure_migrations_table(pool, db_config).await?;
    let dir = migrations_dir(db_config);
    let files = read_migration_files(&dir)?;
    let applied = get_applied_migrations(pool, db_config).await?;

    for (name, _) in &files {
        let status = if applied.contains(name) { "已执行" } else { "待执行" };
        tracing::info!(migration = %name, status = status);
    }
    if files.is_empty() {
        tracing::info!("没有迁移文件");
    }
    Ok(())
}

fn run_create(name: &str, db_config: &tsangwu_config::DatabaseConfig) -> Result<()> {
    let dir = migrations_dir(db_config);
    std::fs::create_dir_all(&dir)?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let up_file = format!("{}/{}_{}.sql", dir, timestamp, name);
    let down_file = format!("{}/{}_{}.down.sql", dir, timestamp, name);
    std::fs::write(&up_file, "-- 迁移 SQL\n")?;
    std::fs::write(&down_file, "-- 回滚 SQL\n")?;
    tracing::info!(up = %up_file, down = %down_file, "迁移文件已创建");
    Ok(())
}

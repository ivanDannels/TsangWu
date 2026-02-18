use async_trait::async_trait;
use serde_json::Value;

/// 文档存储抽象（支持 MongoDB / PG JSONB / SQLite JSON）
#[async_trait]
pub trait DocumentStore: Send + Sync {
    async fn insert(&self, collection: &str, doc: &Value) -> anyhow::Result<String>;
    async fn find_one(&self, collection: &str, filter: &Value) -> anyhow::Result<Option<Value>>;
    async fn find_many(&self, collection: &str, filter: &Value, limit: u64) -> anyhow::Result<Vec<Value>>;
    async fn update_one(&self, collection: &str, filter: &Value, update: &Value) -> anyhow::Result<bool>;
    async fn delete_one(&self, collection: &str, filter: &Value) -> anyhow::Result<bool>;
}

/// PG JSONB 降级实现
/// 使用 documents 表: (id BIGSERIAL, collection VARCHAR, data JSONB, created_at TIMESTAMPTZ)
#[cfg(feature = "postgres")]
pub struct PgDocStore {
    pool: sqlx::PgPool,
}

#[cfg(feature = "postgres")]
impl PgDocStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl DocumentStore for PgDocStore {
    async fn insert(&self, collection: &str, doc: &Value) -> anyhow::Result<String> {
        let row = sqlx::query_scalar::<_, i64>(
            "INSERT INTO documents (collection, data) VALUES ($1, $2) RETURNING id"
        )
        .bind(collection)
        .bind(doc)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.to_string())
    }

    async fn find_one(&self, collection: &str, filter: &Value) -> anyhow::Result<Option<Value>> {
        let row = sqlx::query_scalar::<_, Value>(
            "SELECT data FROM documents WHERE collection = $1 AND data @> $2 LIMIT 1"
        )
        .bind(collection)
        .bind(filter)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn find_many(&self, collection: &str, filter: &Value, limit: u64) -> anyhow::Result<Vec<Value>> {
        let rows = sqlx::query_scalar::<_, Value>(
            "SELECT data FROM documents WHERE collection = $1 AND data @> $2 LIMIT $3"
        )
        .bind(collection)
        .bind(filter)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn update_one(&self, collection: &str, filter: &Value, update: &Value) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "UPDATE documents SET data = data || $3, updated_at = now() WHERE collection = $1 AND data @> $2"
        )
        .bind(collection)
        .bind(filter)
        .bind(update)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_one(&self, collection: &str, filter: &Value) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM documents WHERE collection = $1 AND data @> $2"
        )
        .bind(collection)
        .bind(filter)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

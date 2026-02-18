use async_trait::async_trait;
use std::time::Duration;

/// 缓存后端抽象
#[async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> anyhow::Result<()>;
    async fn del(&self, key: &str) -> anyhow::Result<()>;
    async fn exists(&self, key: &str) -> anyhow::Result<bool>;
    async fn incr(&self, key: &str, ttl_secs: u64) -> anyhow::Result<u64>;
}

/// 内存缓存实现（本地模式）
#[cfg(feature = "in-memory")]
pub mod memory_impl {
    use super::*;
    use moka::future::Cache;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    pub struct MemoryCache {
        inner: Cache<String, Vec<u8>>,
        counters: RwLock<HashMap<String, AtomicU64>>,
    }

    impl MemoryCache {
        pub fn new(max_capacity: u64) -> Self {
            Self {
                inner: Cache::new(max_capacity),
                counters: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CacheBackend for MemoryCache {
        async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
            Ok(self.inner.get(key).await)
        }

        async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> anyhow::Result<()> {
            if let Some(_ttl) = ttl {
                // moka supports TTL per-entry via policy
                self.inner.insert(key.to_string(), value.to_vec()).await;
            } else {
                self.inner.insert(key.to_string(), value.to_vec()).await;
            }
            Ok(())
        }

        async fn del(&self, key: &str) -> anyhow::Result<()> {
            self.inner.remove(key).await;
            Ok(())
        }

        async fn exists(&self, key: &str) -> anyhow::Result<bool> {
            Ok(self.inner.contains_key(key))
        }

        async fn incr(&self, key: &str, _ttl_secs: u64) -> anyhow::Result<u64> {
            let mut counters = self.counters.write().await;
            let counter = counters
                .entry(key.to_string())
                .or_insert_with(|| AtomicU64::new(0));
            let val = counter.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(val)
        }
    }
}

/// Redis 缓存实现
#[cfg(feature = "redis")]
pub mod redis_impl {
    use super::*;
    use fred::prelude::*;

    pub struct RedisCache {
        client: RedisClient,
    }

    impl RedisCache {
        pub async fn new(url: &str) -> anyhow::Result<Self> {
            let config = RedisConfig::from_url(url)?;
            let client = RedisClient::new(config, None, None, None);
            client.connect();
            client.wait_for_connect().await?;
            Ok(Self { client })
        }
    }

    #[async_trait]
    impl CacheBackend for RedisCache {
        async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
            let val: Option<bytes::Bytes> = self.client.get(key).await?;
            Ok(val.map(|b| b.to_vec()))
        }

        async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> anyhow::Result<()> {
            let expiration = ttl.map(|d| Expiration::EX(d.as_secs() as i64));
            let _: () = self.client.set(key, value.to_vec(), expiration, None, false).await?;
            Ok(())
        }

        async fn del(&self, key: &str) -> anyhow::Result<()> {
            let _: u32 = self.client.del(key).await?;
            Ok(())
        }

        async fn exists(&self, key: &str) -> anyhow::Result<bool> {
            let count: i64 = self.client.exists(key).await?;
            Ok(count > 0)
        }

        async fn incr(&self, key: &str, ttl_secs: u64) -> anyhow::Result<u64> {
            let val: u64 = self.client.incr(key).await?;
            let _: bool = self.client.expire(key, ttl_secs as i64).await?;
            Ok(val)
        }
    }
}

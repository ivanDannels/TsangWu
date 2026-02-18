use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub topic: String,
    pub key: String,
    pub payload: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 消息生产者
#[async_trait]
pub trait MessageProducer: Send + Sync {
    async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> anyhow::Result<()>;
}

/// 消息消费者
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    async fn subscribe(
        &self,
        topic: &str,
        group: &str,
        handler: Box<dyn Fn(Message) -> futures::future::BoxFuture<'static, anyhow::Result<()>> + Send + Sync>,
    ) -> anyhow::Result<()>;
}

/// 内存消息队列实现（本地模式）
#[cfg(feature = "in-memory")]
pub mod memory_impl {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::{broadcast, RwLock};

    pub struct InMemoryBroker {
        topics: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>>,
        channel_size: usize,
    }

    impl InMemoryBroker {
        pub fn new(channel_size: usize) -> Self {
            Self {
                topics: Arc::new(RwLock::new(HashMap::new())),
                channel_size,
            }
        }

        async fn get_or_create_topic(&self, topic: &str) -> broadcast::Sender<Message> {
            let mut topics = self.topics.write().await;
            topics
                .entry(topic.to_string())
                .or_insert_with(|| broadcast::channel(self.channel_size).0)
                .clone()
        }
    }

    #[async_trait]
    impl MessageProducer for InMemoryBroker {
        async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> anyhow::Result<()> {
            let sender = self.get_or_create_topic(topic).await;
            let msg = Message {
                topic: topic.to_string(),
                key: key.to_string(),
                payload: payload.to_vec(),
                timestamp: chrono::Utc::now(),
            };
            let _ = sender.send(msg); // ignore if no receivers
            Ok(())
        }
    }

    #[async_trait]
    impl MessageConsumer for InMemoryBroker {
        async fn subscribe(
            &self,
            topic: &str,
            _group: &str,
            handler: Box<dyn Fn(Message) -> futures::future::BoxFuture<'static, anyhow::Result<()>> + Send + Sync>,
        ) -> anyhow::Result<()> {
            let sender = self.get_or_create_topic(topic).await;
            let mut rx = sender.subscribe();
            tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    if let Err(e) = handler(msg).await {
                        tracing::error!("消息处理失败: {}", e);
                    }
                }
            });
            Ok(())
        }
    }
}

/// Kafka 实现
#[cfg(feature = "kafka")]
pub mod kafka_impl {
    use super::*;

    pub struct KafkaProducer {
        _brokers: Vec<String>,
    }

    impl KafkaProducer {
        pub fn new(brokers: Vec<String>) -> anyhow::Result<Self> {
            tracing::info!(brokers = ?brokers, "Kafka 生产者已初始化（模拟模式）");
            Ok(Self { _brokers: brokers })
        }
    }

    #[async_trait]
    impl MessageProducer for KafkaProducer {
        async fn publish(&self, topic: &str, key: &str, _payload: &[u8]) -> anyhow::Result<()> {
            tracing::info!(topic = %topic, key = %key, "Kafka 消息发送（模拟模式）");
            Ok(())
        }
    }
}

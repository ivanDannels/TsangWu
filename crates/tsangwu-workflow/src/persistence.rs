use async_trait::async_trait;
use uuid::Uuid;

/// 工作流持久化 trait
#[async_trait]
pub trait WorkflowPersistence: Send + Sync {
    async fn create_instance(&self, instance: &WorkflowInstanceRecord) -> anyhow::Result<()>;
    async fn get_instance(&self, instance_id: Uuid) -> anyhow::Result<Option<WorkflowInstanceRecord>>;
    async fn update_instance_state(&self, instance_id: Uuid, state: &str, status: i16) -> anyhow::Result<()>;
    async fn record_event(&self, event: &WorkflowEventRecord) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct WorkflowInstanceRecord {
    pub instance_id: Uuid,
    pub definition_name: String,
    pub correlation_id: String,
    pub current_state: String,
    pub status: i16,
    pub context_json: serde_json::Value,
    pub retry_count: i32,
}

#[derive(Debug, Clone)]
pub struct WorkflowEventRecord {
    pub instance_id: Uuid,
    pub event_type: String,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub step_name: Option<String>,
    pub payload_json: Option<serde_json::Value>,
}

use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement, DbBackend};
use uuid::Uuid;

use crate::persistence::{WorkflowEventRecord, WorkflowInstanceRecord, WorkflowPersistence};

/// 基于 PostgreSQL 的工作流持久化实现
pub struct DbWorkflowPersistence {
    db: DatabaseConnection,
}

impl DbWorkflowPersistence {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl WorkflowPersistence for DbWorkflowPersistence {
    async fn create_instance(&self, instance: &WorkflowInstanceRecord) -> anyhow::Result<()> {
        let sql = r#"
            INSERT INTO workflow_instances (instance_id, definition_name, correlation_id, current_state, status, context_json, retry_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                instance.instance_id.to_string().into(),
                instance.definition_name.clone().into(),
                instance.correlation_id.clone().into(),
                instance.current_state.clone().into(),
                instance.status.into(),
                instance.context_json.to_string().into(),
                instance.retry_count.into(),
            ],
        );
        self.db.execute(stmt).await?;
        Ok(())
    }

    async fn get_instance(&self, instance_id: Uuid) -> anyhow::Result<Option<WorkflowInstanceRecord>> {
        let sql = r#"
            SELECT instance_id, definition_name, correlation_id, current_state, status, context_json, retry_count
            FROM workflow_instances WHERE instance_id = $1
        "#;
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![instance_id.to_string().into()],
        );
        let row = self.db.query_one(stmt).await?;
        match row {
            Some(row) => {
                let id_str: String = row.try_get("", "instance_id")?;
                let instance = WorkflowInstanceRecord {
                    instance_id: Uuid::parse_str(&id_str)?,
                    definition_name: row.try_get("", "definition_name")?,
                    correlation_id: row.try_get("", "correlation_id")?,
                    current_state: row.try_get("", "current_state")?,
                    status: row.try_get("", "status")?,
                    context_json: {
                        let s: String = row.try_get("", "context_json")?;
                        serde_json::from_str(&s).unwrap_or_default()
                    },
                    retry_count: row.try_get("", "retry_count")?,
                };
                Ok(Some(instance))
            }
            None => Ok(None),
        }
    }

    async fn update_instance_state(&self, instance_id: Uuid, state: &str, status: i16) -> anyhow::Result<()> {
        let sql = r#"
            UPDATE workflow_instances SET current_state = $1, status = $2 WHERE instance_id = $3
        "#;
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                state.into(),
                status.into(),
                instance_id.to_string().into(),
            ],
        );
        self.db.execute(stmt).await?;
        Ok(())
    }

    async fn record_event(&self, event: &WorkflowEventRecord) -> anyhow::Result<()> {
        let sql = r#"
            INSERT INTO workflow_events (instance_id, event_type, from_state, to_state, step_name, payload_json)
            VALUES ($1, $2, $3, $4, $5, $6)
        "#;
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                event.instance_id.to_string().into(),
                event.event_type.clone().into(),
                event.from_state.clone().unwrap_or_default().into(),
                event.to_state.clone().unwrap_or_default().into(),
                event.step_name.clone().unwrap_or_default().into(),
                event.payload_json.as_ref().map(|v| v.to_string()).unwrap_or_default().into(),
            ],
        );
        self.db.execute(stmt).await?;
        Ok(())
    }
}

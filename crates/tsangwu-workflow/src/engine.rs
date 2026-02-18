use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::definition::{StateType, WorkflowDefinition};
use crate::persistence::{WorkflowEventRecord, WorkflowInstanceRecord, WorkflowPersistence};

/// 步骤处理器 trait
#[async_trait]
pub trait StepHandler: Send + Sync {
    async fn execute(&self, ctx: &mut StepContext) -> Result<StepOutput, StepError>;
    async fn compensate(&self, ctx: &mut StepContext) -> Result<(), StepError> {
        let _ = ctx;
        Ok(())
    }
}

/// 步骤上下文
pub struct StepContext {
    pub instance_id: Uuid,
    pub step_name: String,
    pub input: serde_json::Value,
    pub workflow_context: serde_json::Value,
}

/// 步骤输出
#[derive(Debug)]
pub struct StepOutput {
    pub data: serde_json::Value,
}

/// 步骤错误
#[derive(Debug, thiserror::Error)]
pub enum StepError {
    #[error("步骤执行失败: {0}")]
    ExecutionFailed(String),
    #[error("步骤超时")]
    Timeout,
    #[error("需要重试")]
    Retryable(String),
    #[error("内部错误: {0}")]
    Internal(#[from] anyhow::Error),
}

/// 步骤处理器注册表
pub struct StepHandlerRegistry {
    handlers: HashMap<String, Arc<dyn StepHandler>>,
}

impl StepHandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, handler: Arc<dyn StepHandler>) {
        self.handlers.insert(name.to_string(), handler);
    }

    pub fn get(&self, name: &str) -> Result<Arc<dyn StepHandler>, StepError> {
        self.handlers
            .get(name)
            .cloned()
            .ok_or_else(|| StepError::ExecutionFailed(format!("未找到步骤处理器: {}", name)))
    }
}

impl Default for StepHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 工作流引擎
pub struct WorkflowEngine {
    registry: Arc<StepHandlerRegistry>,
    definitions: HashMap<String, WorkflowDefinition>,
    persistence: Arc<dyn WorkflowPersistence>,
}

impl WorkflowEngine {
    pub fn new(registry: StepHandlerRegistry, persistence: Arc<dyn WorkflowPersistence>) -> Self {
        Self {
            registry: Arc::new(registry),
            definitions: HashMap::new(),
            persistence,
        }
    }

    /// 注册工作流定义
    pub fn register_definition(&mut self, def: WorkflowDefinition) {
        self.definitions.insert(def.name.clone(), def);
    }

    /// 启动新工作流实例
    pub async fn start_workflow(
        &self,
        definition_name: &str,
        correlation_id: &str,
        input: serde_json::Value,
    ) -> anyhow::Result<Uuid> {
        let def = self
            .definitions
            .get(definition_name)
            .ok_or_else(|| anyhow::anyhow!("未找到工作流定义: {}", definition_name))?
            .clone();

        let instance_id = Uuid::new_v4();
        let initial_state = def.initial_state.clone();

        let record = WorkflowInstanceRecord {
            instance_id,
            definition_name: definition_name.to_string(),
            correlation_id: correlation_id.to_string(),
            current_state: initial_state.clone(),
            status: 0, // 0 = running
            context_json: input.clone(),
            retry_count: 0,
        };
        self.persistence.create_instance(&record).await?;

        self.persistence.record_event(&WorkflowEventRecord {
            instance_id,
            event_type: "started".to_string(),
            from_state: None,
            to_state: Some(initial_state.clone()),
            step_name: None,
            payload_json: Some(input),
        }).await?;

        tracing::info!(
            instance_id = %instance_id,
            definition = definition_name,
            "工作流实例已创建"
        );

        self.process_state(instance_id, &def, &initial_state).await?;

        Ok(instance_id)
    }

    /// 向工作流发送外部事件（如用户审批）
    pub async fn send_event(
        &self,
        instance_id: Uuid,
        event: &str,
        payload: serde_json::Value,
    ) -> anyhow::Result<()> {
        let record = self.persistence.get_instance(instance_id).await?
            .ok_or_else(|| anyhow::anyhow!("未找到工作流实例: {}", instance_id))?;

        let def = self.definitions.get(&record.definition_name)
            .ok_or_else(|| anyhow::anyhow!("未找到工作流定义: {}", record.definition_name))?
            .clone();

        let state_def = def.states.get(&record.current_state)
            .ok_or_else(|| anyhow::anyhow!("未找到状态定义: {}", record.current_state))?;

        let transition = state_def.transitions.iter()
            .find(|t| t.event == event)
            .ok_or_else(|| anyhow::anyhow!("当前状态 {} 无法处理事件: {}", record.current_state, event))?;

        let from_state = record.current_state.clone();
        let to_state = transition.target_state.clone();

        self.persistence.update_instance_state(instance_id, &to_state, 0).await?;

        self.persistence.record_event(&WorkflowEventRecord {
            instance_id,
            event_type: format!("event:{}", event),
            from_state: Some(from_state),
            to_state: Some(to_state.clone()),
            step_name: None,
            payload_json: Some(payload),
        }).await?;

        tracing::info!(instance_id = %instance_id, event = event, to = %to_state, "工作流状态转换");

        self.process_state(instance_id, &def, &to_state).await?;

        Ok(())
    }

    /// 根据状态类型驱动执行
    async fn process_state(
        &self,
        instance_id: Uuid,
        def: &WorkflowDefinition,
        state_name: &str,
    ) -> anyhow::Result<()> {
        let state_def = def.states.get(state_name)
            .ok_or_else(|| anyhow::anyhow!("未找到状态定义: {}", state_name))?;

        match state_def.state_type {
            StateType::Processing | StateType::Initial => {
                // 顺序执行所有步骤
                for step in &state_def.steps {
                    self.execute_step(instance_id, step.name.clone(), step.handler.clone()).await?;
                }
                // 自动触发第一个 transition（如果有 "done" 事件）
                if let Some(t) = state_def.transitions.iter().find(|t| t.event == "done") {
                    let to = t.target_state.clone();
                    self.persistence.update_instance_state(instance_id, &to, 0).await?;
                    self.persistence.record_event(&WorkflowEventRecord {
                        instance_id,
                        event_type: "auto:done".to_string(),
                        from_state: Some(state_name.to_string()),
                        to_state: Some(to.clone()),
                        step_name: None,
                        payload_json: None,
                    }).await?;
                    // 递归驱动下一个状态
                    Box::pin(self.process_state(instance_id, def, &to)).await?;
                }
            }
            StateType::Parallel => {
                // 并行执行所有步骤
                let mut handles = Vec::new();
                for step in &state_def.steps {
                    let registry = self.registry.clone();
                    let sid = instance_id;
                    let step_name = step.name.clone();
                    let handler_name = step.handler.clone();
                    handles.push(tokio::spawn(async move {
                        let handler = registry.get(&handler_name)?;
                        let mut ctx = StepContext {
                            instance_id: sid,
                            step_name: step_name.clone(),
                            input: serde_json::Value::Null,
                            workflow_context: serde_json::Value::Null,
                        };
                        handler.execute(&mut ctx).await
                    }));
                }
                for h in handles {
                    h.await??;
                }
                if let Some(t) = state_def.transitions.iter().find(|t| t.event == "done") {
                    let to = t.target_state.clone();
                    self.persistence.update_instance_state(instance_id, &to, 0).await?;
                    Box::pin(self.process_state(instance_id, def, &to)).await?;
                }
            }
            StateType::Terminal => {
                // 终态，标记完成
                self.persistence.update_instance_state(instance_id, state_name, 1).await?;
                self.persistence.record_event(&WorkflowEventRecord {
                    instance_id,
                    event_type: "completed".to_string(),
                    from_state: Some(state_name.to_string()),
                    to_state: None,
                    step_name: None,
                    payload_json: None,
                }).await?;
                tracing::info!(instance_id = %instance_id, state = state_name, "工作流已完成");
            }
            StateType::WaitForEvent => {
                // 挂起等待外部事件
                tracing::info!(instance_id = %instance_id, state = state_name, "工作流等待外部事件");
            }
            _ => {
                tracing::warn!(instance_id = %instance_id, state = state_name, "未处理的状态类型");
            }
        }
        Ok(())
    }

    /// 执行单个步骤
    async fn execute_step(
        &self,
        instance_id: Uuid,
        step_name: String,
        handler_name: String,
    ) -> Result<StepOutput, StepError> {
        let handler = self.registry.get(&handler_name)?;
        let mut ctx = StepContext {
            instance_id,
            step_name: step_name.clone(),
            input: serde_json::Value::Null,
            workflow_context: serde_json::Value::Null,
        };

        tracing::info!(instance_id = %instance_id, step = %step_name, "开始执行步骤");
        let output = handler.execute(&mut ctx).await?;

        let _ = self.persistence.record_event(&WorkflowEventRecord {
            instance_id,
            event_type: "step_completed".to_string(),
            from_state: None,
            to_state: None,
            step_name: Some(step_name.clone()),
            payload_json: Some(output.data.clone()),
        }).await;

        tracing::info!(instance_id = %instance_id, step = %step_name, "步骤执行完成");
        Ok(output)
    }
}

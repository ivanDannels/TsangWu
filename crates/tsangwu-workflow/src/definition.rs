use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 工作流定义（Rust DSL）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub version: u32,
    pub initial_state: String,
    pub states: HashMap<String, StateDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDefinition {
    pub state_type: StateType,
    pub steps: Vec<StepDefinition>,
    pub transitions: Vec<Transition>,
    pub on_error: Option<ErrorHandler>,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StateType {
    Initial,
    Processing,
    WaitForEvent,
    Parallel,
    Decision,
    Compensation,
    Terminal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDefinition {
    pub name: String,
    pub handler: String,
    pub retry_policy: RetryPolicy,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub event: String,
    pub target_state: String,
    pub condition: Option<String>,
}

impl Transition {
    pub fn on(event: &str, target: &str) -> Self {
        Self {
            event: event.into(),
            target_state: target.into(),
            condition: None,
        }
    }

    pub fn on_condition(event: &str, target: &str, condition: &str) -> Self {
        Self {
            event: event.into(),
            target_state: target.into(),
            condition: Some(condition.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_interval: Duration,
    pub backoff_multiplier: f64,
    pub max_interval: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_interval: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            max_interval: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandler {
    pub strategy: ErrorStrategy,
    pub target_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorStrategy {
    Retry,
    Compensate,
    FailWorkflow,
    GoToState,
}

/// 辅助函数：快速创建步骤定义
pub fn step(name: &str, handler: &str, max_retries: u32, timeout_secs: u64) -> StepDefinition {
    StepDefinition {
        name: name.into(),
        handler: handler.into(),
        retry_policy: RetryPolicy {
            max_retries,
            ..Default::default()
        },
        timeout: Some(Duration::from_secs(timeout_secs)),
    }
}

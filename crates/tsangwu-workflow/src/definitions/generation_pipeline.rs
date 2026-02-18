use std::collections::HashMap;
use std::time::Duration;

use crate::definition::{
    ErrorHandler, ErrorStrategy, StateDefinition, StateType,
    Transition, WorkflowDefinition, step,
};

/// 构建生成管线工作流定义
/// 状态流转: parsing → planning → generating → editing → reviewing → compliance_check → completed
pub fn build_generation_pipeline() -> WorkflowDefinition {
    let mut states = HashMap::new();

    // parsing: 解析输入
    states.insert("parsing".to_string(), StateDefinition {
        state_type: StateType::Initial,
        steps: vec![step("parse_input", "parse_input_handler", 3, 60)],
        transitions: vec![Transition::on("done", "planning")],
        on_error: Some(ErrorHandler {
            strategy: ErrorStrategy::Retry,
            target_state: None,
        }),
        timeout: Some(Duration::from_secs(120)),
    });

    // planning: 规划生成策略
    states.insert("planning".to_string(), StateDefinition {
        state_type: StateType::Processing,
        steps: vec![step("plan_generation", "plan_generation_handler", 2, 120)],
        transitions: vec![Transition::on("done", "generating")],
        on_error: Some(ErrorHandler {
            strategy: ErrorStrategy::Retry,
            target_state: None,
        }),
        timeout: Some(Duration::from_secs(300)),
    });

    // generating: 执行生成（并行）
    states.insert("generating".to_string(), StateDefinition {
        state_type: StateType::Parallel,
        steps: vec![
            step("generate_content", "generate_content_handler", 3, 600),
            step("generate_metadata", "generate_metadata_handler", 2, 120),
        ],
        transitions: vec![Transition::on("done", "editing")],
        on_error: Some(ErrorHandler {
            strategy: ErrorStrategy::Retry,
            target_state: None,
        }),
        timeout: Some(Duration::from_secs(900)),
    });

    // editing: 编辑润色
    states.insert("editing".to_string(), StateDefinition {
        state_type: StateType::Processing,
        steps: vec![step("edit_content", "edit_content_handler", 2, 300)],
        transitions: vec![Transition::on("done", "reviewing")],
        on_error: Some(ErrorHandler {
            strategy: ErrorStrategy::Retry,
            target_state: None,
        }),
        timeout: Some(Duration::from_secs(600)),
    });

    // reviewing: 人工审核（等待外部事件）
    states.insert("reviewing".to_string(), StateDefinition {
        state_type: StateType::WaitForEvent,
        steps: vec![],
        transitions: vec![
            Transition::on("approved", "compliance_check"),
            Transition::on("rejected", "editing"),
        ],
        on_error: None,
        timeout: Some(Duration::from_secs(86400)),
    });

    // compliance_check: 合规检查
    states.insert("compliance_check".to_string(), StateDefinition {
        state_type: StateType::Processing,
        steps: vec![step("check_compliance", "check_compliance_handler", 2, 120)],
        transitions: vec![Transition::on("done", "completed")],
        on_error: Some(ErrorHandler {
            strategy: ErrorStrategy::FailWorkflow,
            target_state: None,
        }),
        timeout: Some(Duration::from_secs(300)),
    });

    // completed: 终态
    states.insert("completed".to_string(), StateDefinition {
        state_type: StateType::Terminal,
        steps: vec![],
        transitions: vec![],
        on_error: None,
        timeout: None,
    });

    WorkflowDefinition {
        name: "generation_pipeline".to_string(),
        version: 1,
        initial_state: "parsing".to_string(),
        states,
    }
}

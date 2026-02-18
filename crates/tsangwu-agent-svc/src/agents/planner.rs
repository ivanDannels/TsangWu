use async_trait::async_trait;
use crate::{Agent, AgentContext, AgentOutput, AgentError};

pub struct PlannerAgent;

#[async_trait]
impl Agent for PlannerAgent {
    fn name(&self) -> &str { "planner" }

    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
        tracing::info!(task_id = %ctx.task_id, "创意规划 Agent 执行");

        let dynasty = ctx.shared.dynasty.clone().unwrap_or_default();
        let labels = &ctx.shared.cultural_labels;
        let style = ctx.shared.style_profile.clone();

        // 根据文化标签生成场景计划
        let mut scenes = Vec::new();

        // 场景1: 开场
        scenes.push(serde_json::json!({
            "scene_index": 0,
            "name": "开场",
            "description": format!("{}风格开场画面", if dynasty.is_empty() { "古典" } else { &dynasty }),
            "duration_secs": 3.0,
            "camera": "wide_shot",
            "elements": labels,
        }));

        // 场景2: 主体
        scenes.push(serde_json::json!({
            "scene_index": 1,
            "name": "主体",
            "description": format!("核心内容展示，融合{}文化元素", if dynasty.is_empty() { "传统" } else { &dynasty }),
            "duration_secs": 5.0,
            "camera": "medium_shot",
            "elements": labels,
        }));

        // 场景3: 收尾
        scenes.push(serde_json::json!({
            "scene_index": 2,
            "name": "收尾",
            "description": "意境收束，留白处理",
            "duration_secs": 2.0,
            "camera": "fade_out",
            "elements": [],
        }));

        let plan = serde_json::json!({
            "dynasty": dynasty,
            "style": style,
            "total_scenes": scenes.len(),
            "total_duration_secs": 10.0,
            "scenes": scenes,
            "cultural_labels": labels,
        });

        Ok(AgentOutput {
            data: plan,
            artifacts: vec![],
            metadata: serde_json::json!({ "planner_version": "0.1.0" }),
        })
    }
}

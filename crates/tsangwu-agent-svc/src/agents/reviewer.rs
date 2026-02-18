use async_trait::async_trait;
use crate::{Agent, AgentContext, AgentOutput, AgentError};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct ReviewerAgent;

/// 基于种子的伪随机 f64 [0, 1)
fn pseudo_random_f64(seed: u64) -> f64 {
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut h);
    (h.finish() % 10000) as f64 / 10000.0
}

#[async_trait]
impl Agent for ReviewerAgent {
    fn name(&self) -> &str { "reviewer" }

    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
        tracing::info!(task_id = %ctx.task_id, "审片校验 Agent 执行");

        // 模拟一致性检查分数 [0.85, 0.95)
        let consistency_score = 0.85 + pseudo_random_f64(ctx.task_id.as_u128() as u64) * 0.10;

        // 模拟质量评估分数 [0.80, 0.95)
        let quality_score = 0.80 + pseudo_random_f64(ctx.instance_id.as_u128() as u64) * 0.15;

        // 综合判断
        let passed = consistency_score >= 0.88 && quality_score >= 0.85;
        let needs_revision = !passed;

        let mut review_notes = Vec::new();
        if consistency_score < 0.88 {
            review_notes.push("一致性分数偏低，建议检查角色外观连续性");
        }
        if quality_score < 0.85 {
            review_notes.push("质量分数偏低，建议增强画面细节");
        }
        if passed {
            review_notes.push("审核通过，质量达标");
        }

        let data = serde_json::json!({
            "passed": passed,
            "needs_revision": needs_revision,
            "consistency_score": consistency_score,
            "quality_score": quality_score,
            "overall_score": (consistency_score + quality_score) / 2.0,
            "review_notes": review_notes,
        });

        Ok(AgentOutput {
            data,
            artifacts: vec![],
            metadata: serde_json::json!({ "reviewer_version": "0.1.0" }),
        })
    }
}

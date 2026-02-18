use async_trait::async_trait;
use crate::{Agent, AgentContext, AgentOutput, AgentError, Artifact};

pub struct GeneratorAgent;

#[async_trait]
impl Agent for GeneratorAgent {
    fn name(&self) -> &str { "generator" }

    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
        tracing::info!(task_id = %ctx.task_id, "内容生成 Agent 执行");

        let scenes = ctx.input["scenes"].as_array();
        let mut artifacts = Vec::new();

        let scene_list = match scenes {
            Some(s) => s.clone(),
            None => vec![serde_json::json!({
                "scene_index": 0,
                "description": "默认场景",
            })],
        };

        for scene in &scene_list {
            let idx = scene["scene_index"].as_u64().unwrap_or(0);
            let desc = scene["description"].as_str().unwrap_or("scene");

            // 模拟图像生成 artifact
            let image_path = format!(
                "/tmp/tswu/{}/scene_{}_image.png",
                ctx.task_id, idx
            );
            artifacts.push(Artifact {
                name: format!("scene_{}_image", idx),
                content_type: "image/png".to_string(),
                storage_path: image_path,
            });

            // 模拟视频片段 artifact
            let video_path = format!(
                "/tmp/tswu/{}/scene_{}_video.mp4",
                ctx.task_id, idx
            );
            artifacts.push(Artifact {
                name: format!("scene_{}_video", idx),
                content_type: "video/mp4".to_string(),
                storage_path: video_path,
            });

            tracing::info!(scene_index = idx, desc = %desc, "生成场景素材（模拟）");
        }

        let data = serde_json::json!({
            "generated_scenes": scene_list.len(),
            "total_artifacts": artifacts.len(),
        });

        Ok(AgentOutput {
            data,
            artifacts,
            metadata: serde_json::json!({ "generator_version": "0.1.0" }),
        })
    }
}

/// 模拟平台客户端（抖音/B站/小红书/微信视频号）
pub struct MockPlatformClient;

impl MockPlatformClient {
    pub async fn publish(&self, platform: &str, task_id: &str) -> serde_json::Value {
        tracing::info!(platform = %platform, task_id = %task_id, "模拟发布到平台");
        let mock_url = format!("https://mock-{}.tsangwu.com/video/{}", platform, task_id);
        serde_json::json!({
            "platform": platform,
            "status": "published",
            "url": mock_url,
            "published_at": chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn check_status(&self, platform: &str, publish_id: &str) -> serde_json::Value {
        tracing::info!(platform = %platform, publish_id = %publish_id, "模拟查询发布状态");
        serde_json::json!({
            "status": "published",
            "views": 0,
            "likes": 0,
        })
    }
}

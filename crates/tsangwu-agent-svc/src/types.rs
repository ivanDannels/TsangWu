use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum AgentType {
    #[serde(rename = "script")]
    Script,
    #[serde(rename = "storyboard")]
    Storyboard,
    #[serde(rename = "visual")]
    Visual,
    #[serde(rename = "audio")]
    Audio,
    #[serde(rename = "quality")]
    Quality,
}

/// Agent状态
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum AgentStatus {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "error")]
    Error,
}

/// Agent配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Agent实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub project_id: Uuid,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub config: AgentConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub task_type: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub status: String,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// 创建Agent请求
#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub project_id: Uuid,
    pub agent_type: AgentType,
    pub config: AgentConfig,
}

/// 执行任务请求
#[derive(Debug, Deserialize)]
pub struct ExecuteTaskRequest {
    pub task_type: String,
    pub input: serde_json::Value,
}

/// 剧本生成请求
#[derive(Debug, Deserialize)]
pub struct GenerateScriptRequest {
    pub theme: String,
    pub style: Option<String>,
    pub duration: Option<u32>,
    pub requirements: Option<String>,
}

/// 剧本生成响应
#[derive(Debug, Serialize)]
pub struct GenerateScriptResponse {
    pub script: String,
    pub scenes: Vec<SceneInfo>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneInfo {
    pub scene_id: u32,
    pub description: String,
    pub duration: f32,
    pub characters: Vec<String>,
}

/// 分镜生成请求
#[derive(Debug, Deserialize)]
pub struct GenerateStoryboardRequest {
    pub script: String,
    pub style: Option<String>,
}

/// 分镜生成响应
#[derive(Debug, Serialize)]
pub struct GenerateStoryboardResponse {
    pub storyboard: Vec<StoryboardFrame>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoryboardFrame {
    pub frame_id: u32,
    pub scene_id: u32,
    pub shot_type: String,
    pub description: String,
    pub camera_angle: String,
    pub duration: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AgentConfig {
        AgentConfig {
            agent_type: AgentType::Script,
            model: "test-model".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            system_prompt: Some("测试提示".to_string()),
            extra: serde_json::json!({}),
        }
    }

    #[test]
    fn test_agent_type_serialization() {
        let agent_type = AgentType::Script;
        let json = serde_json::to_string(&agent_type).unwrap();
        assert_eq!(json, "\"script\"");

        let parsed: AgentType = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, AgentType::Script));
    }

    #[test]
    fn test_agent_status_serialization() {
        let status = AgentStatus::Idle;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"idle\"");

        let parsed: AgentStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, AgentStatus::Idle));
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = create_test_config();
        let json = serde_json::to_string(&config).unwrap();
        
        let parsed: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.model, "test-model");
        assert_eq!(parsed.temperature, Some(0.7));
    }

    #[test]
    fn test_create_agent_request() {
        let req = CreateAgentRequest {
            project_id: Uuid::new_v4(),
            agent_type: AgentType::Visual,
            config: create_test_config(),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CreateAgentRequest = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(parsed.agent_type, AgentType::Visual));
    }

    #[test]
    fn test_execute_task_request() {
        let req = ExecuteTaskRequest {
            task_type: "generate_script".to_string(),
            input: serde_json::json!({
                "theme": "唐朝宫廷",
                "style": "历史剧"
            }),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: ExecuteTaskRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.task_type, "generate_script");
        assert_eq!(parsed.input["theme"], "唐朝宫廷");
    }

    #[test]
    fn test_agent_task_creation() {
        let task = AgentTask {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            task_type: "test_task".to_string(),
            input: serde_json::json!({"test": "value"}),
            output: None,
            status: "pending".to_string(),
            error: None,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };

        let json = serde_json::to_string(&task).unwrap();
        let parsed: AgentTask = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.status, "pending");
        assert!(parsed.output.is_none());
    }

    #[test]
    fn test_scene_info() {
        let scene = SceneInfo {
            scene_id: 1,
            description: "开场场景".to_string(),
            duration: 5.0,
            characters: vec!["主角".to_string(), "配角".to_string()],
        };

        let json = serde_json::to_string(&scene).unwrap();
        let parsed: SceneInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.scene_id, 1);
        assert_eq!(parsed.characters.len(), 2);
    }

    #[test]
    fn test_storyboard_frame() {
        let frame = StoryboardFrame {
            frame_id: 1,
            scene_id: 1,
            shot_type: "wide".to_string(),
            description: "全景镜头".to_string(),
            camera_angle: "eye_level".to_string(),
            duration: 3.0,
        };

        let json = serde_json::to_string(&frame).unwrap();
        let parsed: StoryboardFrame = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.shot_type, "wide");
        assert_eq!(parsed.duration, 3.0);
    }
}

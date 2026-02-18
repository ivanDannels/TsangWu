use crate::{error::*, types::*};
use chrono::Utc;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tsangwu_pybridge::PyBridge;
use uuid::Uuid;

pub struct AgentService {
    db: PgPool,
    pybridge: Arc<PyBridge>,
}

impl AgentService {
    pub fn new(db: PgPool, pybridge: Arc<PyBridge>) -> Self {
        Self { db, pybridge }
    }

    /// 创建Agent
    pub async fn create_agent(&self, req: CreateAgentRequest) -> Result<Agent> {
        let agent = Agent {
            id: Uuid::new_v4(),
            project_id: req.project_id,
            agent_type: req.agent_type,
            status: AgentStatus::Idle,
            config: req.config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO agents (id, project_id, agent_type, status, config, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(agent.id)
        .bind(agent.project_id)
        .bind(format!("{:?}", agent.agent_type).to_lowercase())
        .bind(format!("{:?}", agent.status).to_lowercase())
        .bind(serde_json::to_value(&agent.config)?)
        .bind(agent.created_at)
        .bind(agent.updated_at)
        .execute(&self.db)
        .await?;

        Ok(agent)
    }

    /// 获取Agent
    pub async fn get_agent(&self, agent_id: Uuid) -> Result<Agent> {
        let row = sqlx::query(
            r#"
            SELECT id, project_id, agent_type, status, config, created_at, updated_at
            FROM agents WHERE id = $1
            "#,
        )
        .bind(agent_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AgentError::NotFound(agent_id.to_string()))?;

        let agent_type_str: String = row.get("agent_type");
        let status_str: String = row.get("status");

        Ok(Agent {
            id: row.get("id"),
            project_id: row.get("project_id"),
            agent_type: parse_agent_type(&agent_type_str)?,
            status: parse_agent_status(&status_str)?,
            config: serde_json::from_value(row.get("config"))?,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// 列出项目的所有Agent
    pub async fn list_agents(&self, project_id: Uuid) -> Result<Vec<Agent>> {
        let rows = sqlx::query(
            r#"
            SELECT id, project_id, agent_type, status, config, created_at, updated_at
            FROM agents WHERE project_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.db)
        .await?;

        let mut agents = Vec::new();
        for row in rows {
            let agent_type_str: String = row.get("agent_type");
            let status_str: String = row.get("status");
            let agent_type = parse_agent_type(&agent_type_str)?;

            agents.push(Agent {
                id: row.get("id"),
                project_id: row.get("project_id"),
                agent_type: agent_type.clone(),
                status: parse_agent_status(&status_str)?,
                config: serde_json::from_value(row.get("config")).unwrap_or_else(|_| AgentConfig {
                    agent_type,
                    model: "default".to_string(),
                    temperature: None,
                    max_tokens: None,
                    system_prompt: None,
                    extra: serde_json::json!({}),
                }),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(agents)
    }

    /// 执行任务
    pub async fn execute_task(&self, agent_id: Uuid, req: ExecuteTaskRequest) -> Result<AgentTask> {
        let agent = self.get_agent(agent_id).await?;

        // 检查Agent状态
        if !matches!(agent.status, AgentStatus::Idle) {
            return Err(AgentError::Busy(format!("Agent {} is not idle", agent_id)));
        }

        // 更新状态为运行中
        self.update_agent_status(agent_id, AgentStatus::Running).await?;

        // 创建任务记录
        let task = AgentTask {
            id: Uuid::new_v4(),
            agent_id,
            task_type: req.task_type.clone(),
            input: req.input.clone(),
            output: None,
            status: "running".to_string(),
            error: None,
            created_at: Utc::now(),
            completed_at: None,
        };

        sqlx::query(
            r#"
            INSERT INTO agent_tasks (id, agent_id, task_type, input, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(task.id)
        .bind(task.agent_id)
        .bind(&task.task_type)
        .bind(&task.input)
        .bind(&task.status)
        .bind(task.created_at)
        .execute(&self.db)
        .await?;

        // 执行任务（异步）
        let service = self.clone();
        let task_id = task.id;
        tokio::spawn(async move {
            let result = service.execute_task_internal(&agent, &req).await;
            let _ = service.complete_task(task_id, result).await;
            let _ = service.update_agent_status(agent_id, AgentStatus::Idle).await;
        });

        Ok(task)
    }

    /// 内部任务执行
    async fn execute_task_internal(
        &self,
        agent: &Agent,
        req: &ExecuteTaskRequest,
    ) -> Result<serde_json::Value> {
        match agent.agent_type {
            AgentType::Script => self.execute_script_task(req).await,
            AgentType::Storyboard => self.execute_storyboard_task(req).await,
            AgentType::Visual => self.execute_visual_task(req).await,
            AgentType::Audio => self.execute_audio_task(req).await,
            AgentType::Quality => self.execute_quality_task(req).await,
        }
    }

    async fn execute_script_task(&self, req: &ExecuteTaskRequest) -> Result<serde_json::Value> {
        // 模拟剧本生成
        let theme = req.input.get("theme")
            .and_then(|v| v.as_str())
            .unwrap_or("未指定主题");

        let script = format!("这是一个关于{}的剧本...", theme);
        let scenes = vec![
            SceneInfo {
                scene_id: 1,
                description: "开场场景".to_string(),
                duration: 5.0,
                characters: vec!["主角".to_string()],
            },
        ];

        Ok(serde_json::json!({
            "script": script,
            "scenes": scenes,
        }))
    }

    async fn execute_storyboard_task(&self, _req: &ExecuteTaskRequest) -> Result<serde_json::Value> {
        // 模拟分镜生成
        Ok(serde_json::json!({
            "storyboard": [
                {
                    "frame_id": 1,
                    "scene_id": 1,
                    "shot_type": "wide",
                    "description": "全景镜头",
                    "camera_angle": "eye_level",
                    "duration": 3.0
                }
            ]
        }))
    }

    async fn execute_visual_task(&self, req: &ExecuteTaskRequest) -> Result<serde_json::Value> {
        // 调用PyBridge生成图像
        let prompt = req.input.get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("default prompt");

        let params = tsangwu_pybridge::ImageGenParams {
            prompt: prompt.to_string(),
            negative_prompt: None,
            width: 512,
            height: 512,
            style: None,
            lora: None,
            seed: None,
        };

        let image_data = self.pybridge.generate_image(&params).await
            .map_err(|e| AgentError::PyBridge(e.to_string()))?;

        Ok(serde_json::json!({
            "image_size": image_data.len(),
            "format": "png"
        }))
    }

    async fn execute_audio_task(&self, req: &ExecuteTaskRequest) -> Result<serde_json::Value> {
        // 调用PyBridge生成音频
        let text = req.input.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("默认文本");

        let audio_data = self.pybridge.synthesize_speech(text, "default").await
            .map_err(|e| AgentError::PyBridge(e.to_string()))?;

        Ok(serde_json::json!({
            "audio_size": audio_data.len(),
            "format": "wav"
        }))
    }

    async fn execute_quality_task(&self, req: &ExecuteTaskRequest) -> Result<serde_json::Value> {
        // 调用PyBridge进行质量评估
        let paths = req.input.get("paths")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_else(Vec::new);

        let result = self.pybridge.assess_quality(&paths).await
            .map_err(|e| AgentError::PyBridge(e.to_string()))?;

        Ok(serde_json::json!({
            "score": result.score
        }))
    }

    async fn complete_task(&self, task_id: Uuid, result: Result<serde_json::Value>) -> Result<()> {
        match result {
            Ok(output) => {
                sqlx::query(
                    r#"
                    UPDATE agent_tasks
                    SET output = $1, status = 'completed', completed_at = $2
                    WHERE id = $3
                    "#,
                )
                .bind(output)
                .bind(Utc::now())
                .bind(task_id)
                .execute(&self.db)
                .await?;
            }
            Err(e) => {
                sqlx::query(
                    r#"
                    UPDATE agent_tasks
                    SET error = $1, status = 'failed', completed_at = $2
                    WHERE id = $3
                    "#,
                )
                .bind(e.to_string())
                .bind(Utc::now())
                .bind(task_id)
                .execute(&self.db)
                .await?;
            }
        }

        Ok(())
    }

    async fn update_agent_status(&self, agent_id: Uuid, status: AgentStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agents SET status = $1, updated_at = $2 WHERE id = $3
            "#,
        )
        .bind(format!("{:?}", status).to_lowercase())
        .bind(Utc::now())
        .bind(agent_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// 获取任务详情
    pub async fn get_task(&self, task_id: Uuid) -> Result<AgentTask> {
        let row = sqlx::query(
            r#"
            SELECT id, agent_id, task_type, input, output, status, error, created_at, completed_at
            FROM agent_tasks WHERE id = $1
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AgentError::NotFound(task_id.to_string()))?;

        Ok(AgentTask {
            id: row.get("id"),
            agent_id: row.get("agent_id"),
            task_type: row.get("task_type"),
            input: row.get("input"),
            output: row.get("output"),
            status: row.get("status"),
            error: row.get("error"),
            created_at: row.get("created_at"),
            completed_at: row.get("completed_at"),
        })
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE agent_tasks
            SET status = 'cancelled', completed_at = $1
            WHERE id = $2 AND status = 'running'
            "#,
        )
        .bind(Utc::now())
        .bind(task_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AgentError::NotFound(format!("Task {} not found or not running", task_id)));
        }

        Ok(())
    }

    /// 重试失败的任务
    pub async fn retry_task(&self, task_id: Uuid) -> Result<AgentTask> {
        let row = sqlx::query(
            r#"
            SELECT t.id, t.agent_id, t.task_type, t.input, t.status, a.project_id
            FROM agent_tasks t
            JOIN agents a ON t.agent_id = a.id
            WHERE t.id = $1 AND t.status = 'failed'
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AgentError::NotFound(format!("Task {} not found or not failed", task_id)))?;

        let agent_id: Uuid = row.get("agent_id");
        let task_type: String = row.get("task_type");
        let input: serde_json::Value = row.get("input");

        self.execute_task(agent_id, ExecuteTaskRequest { task_type, input }).await
    }

    /// 列出Agent的所有任务
    pub async fn list_tasks(&self, agent_id: Uuid, limit: Option<i32>) -> Result<Vec<AgentTask>> {
        let limit = limit.unwrap_or(50);
        let rows = sqlx::query(
            r#"
            SELECT id, agent_id, task_type, input, output, status, error, created_at, completed_at
            FROM agent_tasks WHERE agent_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| AgentTask {
                id: row.get("id"),
                agent_id: row.get("agent_id"),
                task_type: row.get("task_type"),
                input: row.get("input"),
                output: row.get("output"),
                status: row.get("status"),
                error: row.get("error"),
                created_at: row.get("created_at"),
                completed_at: row.get("completed_at"),
            })
            .collect())
    }
}

impl Clone for AgentService {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            pybridge: Arc::clone(&self.pybridge),
        }
    }
}

fn parse_agent_type(s: &str) -> Result<AgentType> {
    match s {
        "script" => Ok(AgentType::Script),
        "storyboard" => Ok(AgentType::Storyboard),
        "visual" => Ok(AgentType::Visual),
        "audio" => Ok(AgentType::Audio),
        "quality" => Ok(AgentType::Quality),
        _ => Err(AgentError::InvalidType(s.to_string())),
    }
}

fn parse_agent_status(s: &str) -> Result<AgentStatus> {
    match s {
        "idle" => Ok(AgentStatus::Idle),
        "running" => Ok(AgentStatus::Running),
        "paused" => Ok(AgentStatus::Paused),
        "error" => Ok(AgentStatus::Error),
        _ => Err(AgentError::Internal(format!("Invalid status: {}", s))),
    }
}

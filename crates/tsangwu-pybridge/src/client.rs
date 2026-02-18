use crate::{error::*, types::*};
use serde_json::json;
use std::time::Duration;
use tracing::{debug, warn};

/// PyBridge客户端（模拟实现）
#[derive(Clone)]
pub struct PyBridgeClient {
    base_url: String,
    client: reqwest::Client,
    mock_mode: bool,
}

impl PyBridgeClient {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap();

        Self {
            base_url,
            client,
            mock_mode: true, // 默认使用模拟模式
        }
    }

    pub fn with_mock(mut self, mock: bool) -> Self {
        self.mock_mode = mock;
        self
    }

    /// SD文生图
    pub async fn sd_txt2img(&self, req: SdGenerateRequest) -> Result<SdGenerateResponse> {
        if self.mock_mode {
            return self.mock_sd_generate(req).await;
        }

        let url = format!("{}/sd/txt2img", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .json::<SdGenerateResponse>()
            .await?;

        Ok(resp)
    }

    /// SD图生图
    pub async fn sd_img2img(&self, req: SdGenerateRequest) -> Result<SdGenerateResponse> {
        if self.mock_mode {
            return self.mock_sd_generate(req).await;
        }

        let url = format!("{}/sd/img2img", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .json::<SdGenerateResponse>()
            .await?;

        Ok(resp)
    }

    /// ComfyUI工作流执行
    pub async fn comfy_workflow(&self, req: ComfyWorkflowRequest) -> Result<ComfyWorkflowResponse> {
        if self.mock_mode {
            return self.mock_comfy_workflow(req).await;
        }

        let url = format!("{}/comfy/workflow", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .json::<ComfyWorkflowResponse>()
            .await?;

        Ok(resp)
    }

    /// LLM推理
    pub async fn llm_infer(&self, req: LlmInferRequest) -> Result<LlmInferResponse> {
        if self.mock_mode {
            return self.mock_llm_infer(req).await;
        }

        let url = format!("{}/llm/infer", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .json::<LlmInferResponse>()
            .await?;

        Ok(resp)
    }

    /// 获取模型列表
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        if self.mock_mode {
            return self.mock_list_models().await;
        }

        let url = format!("{}/models", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Vec<ModelInfo>>()
            .await?;

        Ok(resp)
    }

    // ========== 模拟实现 ==========

    async fn mock_sd_generate(&self, req: SdGenerateRequest) -> Result<SdGenerateResponse> {
        debug!("Mock SD generate: prompt={}", req.prompt);

        // 模拟生成延迟
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(SdGenerateResponse {
            images: vec![
                "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string()
            ],
            info: SdGenerateInfo {
                seed: req.seed.unwrap_or(42),
                steps: req.steps,
                cfg_scale: req.cfg_scale,
                sampler: req.sampler.unwrap_or_else(|| "Euler a".to_string()),
                model: req.model.unwrap_or_else(|| "sd_xl_base_1.0".to_string()),
            },
        })
    }

    async fn mock_comfy_workflow(&self, req: ComfyWorkflowRequest) -> Result<ComfyWorkflowResponse> {
        debug!("Mock ComfyUI workflow execution");

        tokio::time::sleep(Duration::from_millis(300)).await;

        Ok(ComfyWorkflowResponse {
            job_id: uuid::Uuid::new_v4().to_string(),
            status: "completed".to_string(),
            outputs: Some(req.inputs),
        })
    }

    async fn mock_llm_infer(&self, req: LlmInferRequest) -> Result<LlmInferResponse> {
        debug!("Mock LLM inference: model={}", req.model);

        tokio::time::sleep(Duration::from_millis(200)).await;

        let content = format!(
            "这是一个模拟的LLM响应。您的问题是：{}",
            req.messages.last().map(|m| m.content.as_str()).unwrap_or("")
        );

        Ok(LlmInferResponse {
            content,
            usage: LlmUsage {
                prompt_tokens: 50,
                completion_tokens: 100,
                total_tokens: 150,
            },
        })
    }

    async fn mock_list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                name: "sd_xl_base_1.0".to_string(),
                model_type: "stable-diffusion".to_string(),
                version: "1.0".to_string(),
                status: "ready".to_string(),
            },
            ModelInfo {
                name: "qwen-7b".to_string(),
                model_type: "llm".to_string(),
                version: "1.5".to_string(),
                status: "ready".to_string(),
            },
        ])
    }
}

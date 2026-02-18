use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SD模型类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SdModelType {
    Txt2Img,
    Img2Img,
    Inpaint,
    ControlNet,
}

/// SD生成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdGenerateRequest {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub cfg_scale: f32,
    pub seed: Option<i64>,
    pub sampler: Option<String>,
    pub model: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// SD生成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdGenerateResponse {
    pub images: Vec<String>, // base64编码的图片
    pub info: SdGenerateInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdGenerateInfo {
    pub seed: i64,
    pub steps: u32,
    pub cfg_scale: f32,
    pub sampler: String,
    pub model: String,
}

/// ComfyUI工作流请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyWorkflowRequest {
    pub workflow: serde_json::Value,
    pub inputs: HashMap<String, serde_json::Value>,
}

/// ComfyUI工作流响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyWorkflowResponse {
    pub job_id: String,
    pub status: String,
    pub outputs: Option<HashMap<String, serde_json::Value>>,
}

/// LLM推理请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInferRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

/// LLM推理响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmInferResponse {
    pub content: String,
    pub usage: LlmUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub model_type: String,
    pub version: String,
    pub status: String,
}

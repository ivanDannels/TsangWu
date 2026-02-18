use async_trait::async_trait;
use crate::{Agent, AgentContext, AgentOutput, AgentError};

pub struct ParserAgent;

/// 朝代关键词
const DYNASTY_KEYWORDS: &[(&str, &str)] = &[
    ("秦", "秦朝"), ("秦", "秦代"), ("秦", "始皇"),
    ("汉", "汉朝"), ("汉", "汉代"), ("汉", "汉服"),
    ("唐", "唐朝"), ("唐", "唐代"), ("唐", "盛唐"), ("唐", "大唐"),
    ("宋", "宋朝"), ("宋", "宋代"), ("宋", "北宋"), ("宋", "南宋"),
    ("元", "元朝"), ("元", "元代"),
    ("明", "明朝"), ("明", "明代"), ("明", "大明"),
    ("清", "清朝"), ("清", "清代"), ("清", "大清"),
];

/// 风格关键词
const STYLE_KEYWORDS: &[&str] = &[
    "工笔", "写意", "水墨", "青绿山水", "白描", "没骨",
];

/// 文化元素关键词
const ELEMENT_KEYWORDS: &[(&str, &str)] = &[
    ("服饰", "汉服"), ("服饰", "唐装"), ("服饰", "旗袍"),
    ("器物", "青花瓷"), ("器物", "唐三彩"), ("器物", "宋瓷"),
    ("建筑", "园林"), ("建筑", "宫殿"), ("建筑", "亭台"),
    ("自然", "山水"), ("自然", "竹林"), ("自然", "梅花"),
];

#[async_trait]
impl Agent for ParserAgent {
    fn name(&self) -> &str { "parser" }

    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
        tracing::info!(task_id = %ctx.task_id, "需求解析 Agent 执行");

        let prompt = ctx.input["prompt"].as_str().unwrap_or("");

        // 识别朝代
        let mut dynasty: Option<String> = None;
        for &(dy, kw) in DYNASTY_KEYWORDS {
            if prompt.contains(kw) {
                dynasty = Some(dy.to_string());
                break;
            }
        }

        // 识别风格
        let mut styles = Vec::new();
        for &kw in STYLE_KEYWORDS {
            if prompt.contains(kw) {
                styles.push(kw.to_string());
            }
        }

        // 识别文化元素
        let mut cultural_labels = Vec::new();
        let mut elements = Vec::new();
        for &(category, keyword) in ELEMENT_KEYWORDS {
            if prompt.contains(keyword) {
                cultural_labels.push(format!("{}:{}", category, keyword));
                elements.push(serde_json::json!({
                    "category": category,
                    "name": keyword,
                }));
            }
        }

        // 更新 shared state
        ctx.shared.dynasty = dynasty.clone();
        ctx.shared.cultural_labels = cultural_labels.clone();
        if !styles.is_empty() {
            ctx.shared.style_profile = serde_json::json!({ "styles": styles });
        }

        let data = serde_json::json!({
            "prompt": prompt,
            "dynasty": dynasty,
            "styles": styles,
            "cultural_labels": cultural_labels,
            "elements": elements,
        });

        Ok(AgentOutput {
            data,
            artifacts: vec![],
            metadata: serde_json::json!({ "parser_version": "0.1.0" }),
        })
    }
}

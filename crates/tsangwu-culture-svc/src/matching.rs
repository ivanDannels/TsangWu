use serde::Serialize;
use crate::knowledge::{ASSET_LIBRARY, DYNASTIES};

/// 文化匹配服务
pub struct CultureMatcher;

#[derive(Debug, Serialize)]
pub struct MatchResult {
    pub assets: Vec<MatchedItem>,
    pub templates: Vec<MatchedItem>,
    pub style_params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct MatchedItem {
    pub id: String,
    pub name: String,
    pub score: f32,
}

impl CultureMatcher {
    pub fn new() -> Self { Self }

    pub async fn match_resources(
        &self,
        labels: &[String],
        dynasty: Option<&str>,
    ) -> anyhow::Result<MatchResult> {
        tracing::info!(labels = ?labels, dynasty = ?dynasty, "文化资源匹配");

        let mut assets = Vec::new();
        let mut templates = Vec::new();

        for entry in ASSET_LIBRARY {
            let mut matches = 0u32;

            // 按标签交集评分
            for label in labels {
                let label_lower = label.to_lowercase();
                for tag in entry.tags {
                    if label_lower.contains(tag) || tag.contains(&label_lower.as_str()) {
                        matches += 1;
                    }
                }
            }

            // 朝代匹配加分
            if let Some(dy) = dynasty {
                if let Some(entry_dy) = entry.dynasty {
                    if entry_dy == dy {
                        matches += 2;
                    }
                }
            }

            if matches > 0 {
                let total_tags = entry.tags.len().max(1) as f32;
                let score = (matches as f32 / total_tags).min(1.0);

                let item = MatchedItem {
                    id: entry.id.to_string(),
                    name: entry.name.to_string(),
                    score,
                };

                match entry.asset_type {
                    "template" => templates.push(item),
                    _ => assets.push(item),
                }
            }
        }

        // 按分数降序排列
        assets.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        templates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // 构建风格参数
        let style_params = if let Some(dy) = dynasty {
            let dynasty_info = DYNASTIES.iter().find(|d| d.name == dy);
            serde_json::json!({
                "dynasty": dy,
                "period": dynasty_info.map(|d| format!("{}-{}", d.start_year, d.end_year)),
                "matched_assets": assets.len(),
                "matched_templates": templates.len(),
            })
        } else {
            serde_json::json!({
                "matched_assets": assets.len(),
                "matched_templates": templates.len(),
            })
        };

        Ok(MatchResult {
            assets,
            templates,
            style_params,
        })
    }
}

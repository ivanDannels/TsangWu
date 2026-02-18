use serde::{Deserialize, Serialize};
use crate::knowledge::{DYNASTIES, STYLES, ELEMENT_KEYWORDS};

/// 文化识别服务
pub struct CultureIdentifier;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CulturalTags {
    pub dynasty: Option<String>,
    pub styles: Vec<String>,
    pub elements: Vec<CulturalElement>,
    pub narrative_type: Option<String>,
    pub color_palette: Vec<String>,
    pub labels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CulturalElement {
    pub category: String,
    pub name: String,
    pub dynasty: Option<String>,
    pub confidence: f32,
}

impl CultureIdentifier {
    pub fn new() -> Self { Self }

    /// 从文本和图片中识别文化元素
    pub async fn identify(&self, text: &str, _images: &[String]) -> anyhow::Result<CulturalTags> {
        tracing::info!(text_len = text.len(), "文化元素识别");

        let mut tags = CulturalTags::default();

        // 扫描朝代关键词
        for dynasty in DYNASTIES {
            for kw in dynasty.keywords {
                if text.contains(kw) {
                    if tags.dynasty.is_none() {
                        tags.dynasty = Some(dynasty.name.to_string());
                    }
                    let label = format!("朝代:{}", dynasty.name);
                    if !tags.labels.contains(&label) {
                        tags.labels.push(label);
                    }
                    break;
                }
            }
        }

        // 扫描风格关键词
        for style in STYLES {
            for kw in style.keywords {
                if text.contains(kw) {
                    if !tags.styles.contains(&style.name.to_string()) {
                        tags.styles.push(style.name.to_string());
                        tags.labels.push(format!("风格:{}", style.name));
                    }
                    break;
                }
            }
        }

        // 扫描文化元素关键词
        for &(category, keyword) in ELEMENT_KEYWORDS {
            if text.contains(keyword) {
                // 尝试关联朝代
                let elem_dynasty = DYNASTIES.iter()
                    .find(|d| d.keywords.iter().any(|k| keyword.contains(k) || k.contains(keyword)))
                    .map(|d| d.name.to_string());

                tags.elements.push(CulturalElement {
                    category: category.to_string(),
                    name: keyword.to_string(),
                    dynasty: elem_dynasty,
                    confidence: 0.85,
                });
                let label = format!("{}:{}", category, keyword);
                if !tags.labels.contains(&label) {
                    tags.labels.push(label);
                }
            }
        }

        // 推断叙事类型
        if text.contains("故事") || text.contains("传说") || text.contains("典故") {
            tags.narrative_type = Some("叙事".to_string());
        } else if text.contains("风景") || text.contains("山水") || text.contains("景色") {
            tags.narrative_type = Some("写景".to_string());
        } else if text.contains("人物") || text.contains("肖像") || text.contains("仕女") {
            tags.narrative_type = Some("人物".to_string());
        }

        // 提取颜色关键词
        let colors = ["朱红", "石青", "石绿", "赭石", "藤黄", "花青",
                       "明黄", "正黄", "墨色", "金色", "银色", "白色"];
        for color in &colors {
            if text.contains(color) {
                tags.color_palette.push(color.to_string());
            }
        }

        Ok(tags)
    }
}

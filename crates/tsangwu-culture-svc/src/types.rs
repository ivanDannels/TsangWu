use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 朝代信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dynasty {
    pub id: Uuid,
    pub name: String,
    pub period: String,
    pub start_year: i32,
    pub end_year: i32,
    pub description: String,
    pub cultural_features: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// 文化元素类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ElementType {
    #[serde(rename = "architecture")]
    Architecture,
    #[serde(rename = "costume")]
    Costume,
    #[serde(rename = "music")]
    Music,
    #[serde(rename = "art")]
    Art,
    #[serde(rename = "literature")]
    Literature,
    #[serde(rename = "custom")]
    Custom,
}

/// 文化元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureElement {
    pub id: Uuid,
    pub dynasty_id: Uuid,
    pub element_type: ElementType,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub reference_images: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// 文化验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub id: Uuid,
    pub dynasty_id: Uuid,
    pub rule_type: String,
    pub description: String,
    pub severity: String, // "error", "warning", "info"
    pub pattern: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 文化验证请求
#[derive(Debug, Deserialize)]
pub struct ValidateContentRequest {
    pub dynasty_id: Uuid,
    pub content_type: String,
    pub content: serde_json::Value,
}

/// 文化验证响应
#[derive(Debug, Serialize)]
pub struct ValidateContentResponse {
    pub valid: bool,
    pub score: f32,
    pub issues: Vec<ValidationIssue>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: String,
    pub message: String,
    pub field: Option<String>,
    pub suggestion: Option<String>,
}

/// 文化推荐请求
#[derive(Debug, Deserialize)]
pub struct RecommendElementsRequest {
    pub dynasty_id: Uuid,
    pub element_type: ElementType,
    pub context: Option<String>,
    pub limit: Option<usize>,
}

/// 文化推荐响应
#[derive(Debug, Serialize)]
pub struct RecommendElementsResponse {
    pub elements: Vec<CultureElement>,
    pub reasoning: String,
}

/// 创建朝代请求
#[derive(Debug, Deserialize)]
pub struct CreateDynastyRequest {
    pub name: String,
    pub period: String,
    pub start_year: i32,
    pub end_year: i32,
    pub description: String,
    pub cultural_features: Vec<String>,
}

/// 创建文化元素请求
#[derive(Debug, Deserialize)]
pub struct CreateElementRequest {
    pub dynasty_id: Uuid,
    pub element_type: ElementType,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub reference_images: Vec<String>,
    pub metadata: serde_json::Value,
}

/// 文化知识图谱节点
#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub properties: serde_json::Value,
}

/// 文化知识图谱关系
#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeRelation {
    pub from_id: String,
    pub to_id: String,
    pub relation_type: String,
    pub properties: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_type_serialization() {
        let element_type = ElementType::Architecture;
        let json = serde_json::to_string(&element_type).unwrap();
        assert_eq!(json, "\"architecture\"");

        let parsed: ElementType = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ElementType::Architecture));
    }

    #[test]
    fn test_dynasty_creation() {
        let dynasty = Dynasty {
            id: Uuid::new_v4(),
            name: "唐朝".to_string(),
            period: "盛唐".to_string(),
            start_year: 618,
            end_year: 907,
            description: "中国历史上的黄金时代".to_string(),
            cultural_features: vec!["诗歌繁荣".to_string(), "丝绸之路".to_string()],
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&dynasty).unwrap();
        let parsed: Dynasty = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.name, "唐朝");
        assert_eq!(parsed.start_year, 618);
        assert_eq!(parsed.cultural_features.len(), 2);
    }

    #[test]
    fn test_culture_element_creation() {
        let element = CultureElement {
            id: Uuid::new_v4(),
            dynasty_id: Uuid::new_v4(),
            element_type: ElementType::Costume,
            name: "唐装".to_string(),
            description: "唐朝传统服饰".to_string(),
            tags: vec!["服饰".to_string(), "宫廷".to_string()],
            reference_images: vec!["http://example.com/image.jpg".to_string()],
            metadata: serde_json::json!({"color": "red"}),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&element).unwrap();
        let parsed: CultureElement = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.name, "唐装");
        assert!(matches!(parsed.element_type, ElementType::Costume));
    }

    #[test]
    fn test_validate_content_request() {
        let req = ValidateContentRequest {
            dynasty_id: Uuid::new_v4(),
            content_type: "script".to_string(),
            content: serde_json::json!({
                "text": "唐朝宫廷剧本内容"
            }),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: ValidateContentRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.content_type, "script");
    }

    #[test]
    fn test_validate_content_response() {
        let response = ValidateContentResponse {
            valid: true,
            score: 0.92,
            issues: vec![ValidationIssue {
                severity: "warning".to_string(),
                message: "建议添加更多唐朝特色元素".to_string(),
                field: Some("scene_1".to_string()),
                suggestion: Some("可以加入大明宫等场景".to_string()),
            }],
            suggestions: vec!["参考唐朝时期的典型文化元素".to_string()],
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: ValidateContentResponse = serde_json::from_str(&json).unwrap();
        
        assert!(parsed.valid);
        assert_eq!(parsed.score, 0.92);
        assert_eq!(parsed.issues.len(), 1);
    }

    #[test]
    fn test_recommend_elements_request() {
        let req = RecommendElementsRequest {
            dynasty_id: Uuid::new_v4(),
            element_type: ElementType::Architecture,
            context: Some("宫廷宴会场景".to_string()),
            limit: Some(10),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: RecommendElementsRequest = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(parsed.element_type, ElementType::Architecture));
        assert_eq!(parsed.limit, Some(10));
    }

    #[test]
    fn test_knowledge_node() {
        let node = KnowledgeNode {
            id: "dynasty_1".to_string(),
            node_type: "dynasty".to_string(),
            name: "唐朝".to_string(),
            properties: serde_json::json!({
                "period": "618-907"
            }),
        };

        let json = serde_json::to_string(&node).unwrap();
        let parsed: KnowledgeNode = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.node_type, "dynasty");
        assert_eq!(parsed.properties["period"], "618-907");
    }

    #[test]
    fn test_knowledge_relation() {
        let relation = KnowledgeRelation {
            from_id: "dynasty_1".to_string(),
            to_id: "element_1".to_string(),
            relation_type: "has_element".to_string(),
            properties: serde_json::json!({}),
        };

        let json = serde_json::to_string(&relation).unwrap();
        let parsed: KnowledgeRelation = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.relation_type, "has_element");
    }
}

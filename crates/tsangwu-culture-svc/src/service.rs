use crate::{error::*, types::*};
use chrono::Utc;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// 缓存键前缀
const CACHE_KEY_DYNASTY: &str = "culture:dynasty:";
const CACHE_KEY_DYNASTIES: &str = "culture:dynasties:all";
const CACHE_KEY_ELEMENTS: &str = "culture:elements:";
const CACHE_TTL_SECS: u64 = 3600; // 1小时

/// 文化引擎服务
pub struct CultureService {
    db: PgPool,
    cache: Option<Arc<dyn tsangwu_cache::CacheBackend>>,
}

impl CultureService {
    /// 创建新的文化引擎服务
    pub fn new(db: PgPool) -> Self {
        Self { db, cache: None }
    }

    /// 创建带缓存的文化引擎服务
    pub fn with_cache(db: PgPool, cache: Arc<dyn tsangwu_cache::CacheBackend>) -> Self {
        Self { db, cache: Some(cache) }
    }

    /// 从缓存获取数据
    async fn get_cache<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        if let Some(cache) = &self.cache {
            if let Ok(Some(data)) = cache.get(key).await {
                if let Ok(value) = serde_json::from_slice(&data) {
                    tracing::debug!(key = %key, "缓存命中");
                    return Some(value);
                }
            }
        }
        None
    }

    /// 设置缓存数据
    async fn set_cache<T: serde::Serialize>(&self, key: &str, value: &T) {
        if let Some(cache) = &self.cache {
            if let Ok(data) = serde_json::to_vec(value) {
                let _ = cache.set(key, &data, Some(Duration::from_secs(CACHE_TTL_SECS))).await;
                tracing::debug!(key = %key, "缓存已设置");
            }
        }
    }

    /// 使缓存失效
    async fn invalidate_cache(&self, key: &str) {
        if let Some(cache) = &self.cache {
            let _ = cache.del(key).await;
            tracing::debug!(key = %key, "缓存已失效");
        }
    }

    /// 创建朝代
    pub async fn create_dynasty(&self, req: CreateDynastyRequest) -> Result<Dynasty> {
        let dynasty = Dynasty {
            id: Uuid::new_v4(),
            name: req.name,
            period: req.period,
            start_year: req.start_year,
            end_year: req.end_year,
            description: req.description,
            cultural_features: req.cultural_features,
            created_at: Utc::now(),
        };

        sqlx::query!(
            r#"
            INSERT INTO dynasties (id, name, period, start_year, end_year, description, cultural_features, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            dynasty.id,
            dynasty.name,
            dynasty.period,
            dynasty.start_year,
            dynasty.end_year,
            dynasty.description,
            &dynasty.cultural_features,
            dynasty.created_at,
        )
        .execute(&self.db)
        .await?;

        // 使朝代列表缓存失效
        self.invalidate_cache(CACHE_KEY_DYNASTIES).await;

        Ok(dynasty)
    }

    /// 获取朝代
    pub async fn get_dynasty(&self, dynasty_id: Uuid) -> Result<Dynasty> {
        let cache_key = format!("{}{}", CACHE_KEY_DYNASTY, dynasty_id);
        
        // 尝试从缓存获取
        if let Some(dynasty) = self.get_cache::<Dynasty>(&cache_key).await {
            return Ok(dynasty);
        }

        let row = sqlx::query!(
            r#"
            SELECT id, name, period, start_year, end_year, description, cultural_features, created_at
            FROM dynasties WHERE id = $1
            "#,
            dynasty_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| CultureError::DynastyNotFound(dynasty_id.to_string()))?;

        let dynasty = Dynasty {
            id: row.id,
            name: row.name,
            period: row.period,
            start_year: row.start_year,
            end_year: row.end_year,
            description: row.description,
            cultural_features: row.cultural_features,
            created_at: row.created_at,
        };

        // 设置缓存
        self.set_cache(&cache_key, &dynasty).await;

        Ok(dynasty)
    }

    /// 列出所有朝代
    pub async fn list_dynasties(&self) -> Result<Vec<Dynasty>> {
        // 尝试从缓存获取
        if let Some(dynasties) = self.get_cache::<Vec<Dynasty>>(CACHE_KEY_DYNASTIES).await {
            return Ok(dynasties);
        }

        let rows = sqlx::query!(
            r#"
            SELECT id, name, period, start_year, end_year, description, cultural_features, created_at
            FROM dynasties
            ORDER BY start_year ASC
            "#
        )
        .fetch_all(&self.db)
        .await?;

        let dynasties: Vec<Dynasty> = rows
            .into_iter()
            .map(|row| Dynasty {
                id: row.id,
                name: row.name,
                period: row.period,
                start_year: row.start_year,
                end_year: row.end_year,
                description: row.description,
                cultural_features: row.cultural_features,
                created_at: row.created_at,
            })
            .collect();

        // 设置缓存
        self.set_cache(CACHE_KEY_DYNASTIES, &dynasties).await;

        Ok(dynasties)
    }

    /// 创建文化元素
    pub async fn create_element(&self, req: CreateElementRequest) -> Result<CultureElement> {
        let element = CultureElement {
            id: Uuid::new_v4(),
            dynasty_id: req.dynasty_id,
            element_type: req.element_type,
            name: req.name,
            description: req.description,
            tags: req.tags,
            reference_images: req.reference_images,
            metadata: req.metadata,
            created_at: Utc::now(),
        };

        sqlx::query!(
            r#"
            INSERT INTO culture_elements (id, dynasty_id, element_type, name, description, tags, reference_images, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            element.id,
            element.dynasty_id,
            element.element_type as ElementType,
            element.name,
            element.description,
            &element.tags,
            &element.reference_images,
            element.metadata,
            element.created_at,
        )
        .execute(&self.db)
        .await?;

        // 使该朝代的元素缓存失效
        let cache_key = format!("{}{}", CACHE_KEY_ELEMENTS, element.dynasty_id);
        self.invalidate_cache(&cache_key).await;

        Ok(element)
    }

    /// 获取文化元素
    pub async fn get_element(&self, element_id: Uuid) -> Result<CultureElement> {
        let row = sqlx::query!(
            r#"
            SELECT id, dynasty_id, element_type as "element_type: ElementType", name, description,
                   tags, reference_images, metadata, created_at
            FROM culture_elements WHERE id = $1
            "#,
            element_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| CultureError::ElementNotFound(element_id.to_string()))?;

        Ok(CultureElement {
            id: row.id,
            dynasty_id: row.dynasty_id,
            element_type: row.element_type,
            name: row.name,
            description: row.description,
            tags: row.tags,
            reference_images: row.reference_images,
            metadata: row.metadata,
            created_at: row.created_at,
        })
    }

    /// 列出朝代的文化元素
    pub async fn list_elements(&self, dynasty_id: Uuid, element_type: Option<ElementType>) -> Result<Vec<CultureElement>> {
        // 构建缓存键（包含元素类型）
        let cache_key = match &element_type {
            Some(et) => format!("{}{}:{:?}", CACHE_KEY_ELEMENTS, dynasty_id, et),
            None => format!("{}{}:all", CACHE_KEY_ELEMENTS, dynasty_id),
        };

        // 尝试从缓存获取
        if let Some(elements) = self.get_cache::<Vec<CultureElement>>(&cache_key).await {
            return Ok(elements);
        }

        let rows = if let Some(et) = element_type {
            sqlx::query!(
                r#"
                SELECT id, dynasty_id, element_type as "element_type: ElementType", name, description,
                       tags, reference_images, metadata, created_at
                FROM culture_elements
                WHERE dynasty_id = $1 AND element_type = $2
                ORDER BY created_at DESC
                "#,
                dynasty_id,
                et as ElementType
            )
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT id, dynasty_id, element_type as "element_type: ElementType", name, description,
                       tags, reference_images, metadata, created_at
                FROM culture_elements
                WHERE dynasty_id = $1
                ORDER BY created_at DESC
                "#,
                dynasty_id
            )
            .fetch_all(&self.db)
            .await?
        };

        let elements: Vec<CultureElement> = rows
            .into_iter()
            .map(|row| CultureElement {
                id: row.id,
                dynasty_id: row.dynasty_id,
                element_type: row.element_type,
                name: row.name,
                description: row.description,
                tags: row.tags,
                reference_images: row.reference_images,
                metadata: row.metadata,
                created_at: row.created_at,
            })
            .collect();

        // 设置缓存
        self.set_cache(&cache_key, &elements).await;

        Ok(elements)
    }

    /// 验证内容的文化准确性
    pub async fn validate_content(&self, req: ValidateContentRequest) -> Result<ValidateContentResponse> {
        // 获取朝代信息
        let dynasty = self.get_dynasty(req.dynasty_id).await?;

        // 模拟文化验证逻辑
        let mut issues = Vec::new();
        let mut score = 1.0f32;

        // 检查内容中的关键词
        let content_str = serde_json::to_string(&req.content)?;

        // 简单的验证规则示例
        if content_str.contains("现代") || content_str.contains("科技") {
            issues.push(ValidationIssue {
                severity: "warning".to_string(),
                message: format!("内容中包含与{}时期不符的现代元素", dynasty.name),
                field: None,
                suggestion: Some("建议使用符合历史时期的描述".to_string()),
            });
            score -= 0.2;
        }

        // 检查是否包含朝代特征
        let has_cultural_feature = dynasty.cultural_features.iter()
            .any(|feature| content_str.contains(feature));

        if !has_cultural_feature {
            issues.push(ValidationIssue {
                severity: "info".to_string(),
                message: format!("建议添加{}时期的文化特征", dynasty.name),
                field: None,
                suggestion: Some(format!("可以考虑加入：{}", dynasty.cultural_features.join("、"))),
            });
            score -= 0.1;
        }

        let valid = score >= 0.6;
        let suggestions = vec![
            format!("参考{}时期的典型文化元素", dynasty.name),
            "确保服饰、建筑等细节符合历史考证".to_string(),
        ];

        Ok(ValidateContentResponse {
            valid,
            score: score.max(0.0),
            issues,
            suggestions,
        })
    }

    /// 推荐文化元素
    pub async fn recommend_elements(&self, req: RecommendElementsRequest) -> Result<RecommendElementsResponse> {
        let limit = req.limit.unwrap_or(10);
        let elements = self.list_elements(req.dynasty_id, Some(req.element_type.clone())).await?;

        let recommended: Vec<CultureElement> = elements.into_iter().take(limit).collect();

        let reasoning = format!(
            "基于{}朝代的{:?}类型元素，推荐了{}个相关文化元素",
            req.dynasty_id,
            req.element_type,
            recommended.len()
        );

        Ok(RecommendElementsResponse {
            elements: recommended,
            reasoning,
        })
    }

    /// 获取文化知识图谱
    pub async fn get_knowledge_graph(&self, dynasty_id: Uuid) -> Result<(Vec<KnowledgeNode>, Vec<KnowledgeRelation>)> {
        // 模拟知识图谱数据
        let dynasty = self.get_dynasty(dynasty_id).await?;
        let elements = self.list_elements(dynasty_id, None).await?;

        let mut nodes = vec![
            KnowledgeNode {
                id: dynasty.id.to_string(),
                node_type: "dynasty".to_string(),
                name: dynasty.name.clone(),
                properties: serde_json::json!({
                    "period": dynasty.period,
                    "years": format!("{}-{}", dynasty.start_year, dynasty.end_year)
                }),
            }
        ];

        let mut relations = Vec::new();

        for element in elements.iter().take(20) {
            nodes.push(KnowledgeNode {
                id: element.id.to_string(),
                node_type: format!("{:?}", element.element_type).to_lowercase(),
                name: element.name.clone(),
                properties: serde_json::json!({
                    "tags": element.tags
                }),
            });

            relations.push(KnowledgeRelation {
                from_id: dynasty.id.to_string(),
                to_id: element.id.to_string(),
                relation_type: "has_element".to_string(),
                properties: serde_json::json!({}),
            });
        }

        Ok((nodes, relations))
    }
}

impl Clone for CultureService {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            cache: self.cache.clone(),
        }
    }
}

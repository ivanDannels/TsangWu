use serde::Serialize;
use crate::knowledge::VALIDATION_RULES;

/// 文化校验引擎
pub struct CultureValidator;

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub issues: Vec<ValidationIssue>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ValidationIssue {
    pub rule_id: String,
    pub category: String,
    pub severity: String,
    pub message: String,
    pub suggestion: String,
}

impl CultureValidator {
    pub fn new() -> Self { Self }

    pub async fn validate(
        &self,
        dynasty: Option<&str>,
        costumes: &[String],
        props: &[String],
    ) -> anyhow::Result<ValidationResult> {
        tracing::info!(dynasty = ?dynasty, "文化校验");

        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        let dynasty_name = match dynasty {
            Some(d) => d,
            None => {
                return Ok(ValidationResult {
                    passed: true,
                    issues: vec![],
                    suggestions: vec!["未指定朝代，跳过朝代相关校验".into()],
                });
            }
        };

        // 遍历规则集进行校验
        for rule in VALIDATION_RULES {
            if rule.dynasty != dynasty_name {
                continue;
            }

            let items: Vec<&String> = match rule.category {
                "服饰" => costumes.iter().collect(),
                "器物" => props.iter().collect(),
                "颜色禁忌" => {
                    // 颜色禁忌同时检查服饰和器物描述
                    costumes.iter().chain(props.iter()).collect()
                }
                _ => vec![],
            };

            for item in &items {
                for &forbidden in rule.forbidden {
                    if item.contains(forbidden) {
                        issues.push(ValidationIssue {
                            rule_id: rule.id.to_string(),
                            category: rule.category.to_string(),
                            severity: "error".to_string(),
                            message: format!("{}: 发现「{}」", rule.message, item),
                            suggestion: format!(
                                "请移除或替换「{}」为{}时期合适的{}",
                                forbidden, dynasty_name, rule.category
                            ),
                        });
                    }
                }
            }
        }

        let passed = issues.is_empty();
        if passed {
            suggestions.push(format!("{}朝代文化校验通过", dynasty_name));
        } else {
            suggestions.push(format!(
                "发现 {} 个文化不一致问题，请根据建议修正",
                issues.len()
            ));
        }

        Ok(ValidationResult {
            passed,
            issues,
            suggestions,
        })
    }
}

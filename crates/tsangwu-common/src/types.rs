use serde::{Deserialize, Serialize};

/// 场景类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneType {
    Image = 0,
    ShortDrama = 1,
    Film = 2,
    Series = 3,
}

/// 用户模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserMode {
    Beginner = 0,
    Advanced = 1,
    Professional = 2,
}

/// 账号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Personal = 0,
    Enterprise = 1,
    ProfessionalTeam = 2,
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending = 0,
    Queued = 1,
    Running = 2,
    Completed = 3,
    Failed = 4,
    Cancelled = 5,
}

/// 素材分类
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetCategory {
    Costume,
    Scene,
    Prop,
    Color,
    Pattern,
    Narrative,
    Storyboard,
    Script,
    Character,
    SceneModel,
    UserUpload,
    UserGenerated,
}

/// 订阅套餐
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanCode {
    Free,
    Personal,
    Advanced,
    Pro,
    Enterprise,
}

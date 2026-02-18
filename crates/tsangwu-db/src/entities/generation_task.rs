use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "generation_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uid: Uuid,
    pub project_id: i64,
    pub user_id: i64,
    pub task_type: i16,
    pub priority: i16,
    pub status: i16,
    pub input_params: serde_json::Value,
    pub output_urls: Option<serde_json::Value>,
    pub progress: Option<i16>,
    pub error_msg: Option<String>,
    pub started_at: Option<DateTimeWithTimeZone>,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

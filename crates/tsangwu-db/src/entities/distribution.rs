use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "distributions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uid: Uuid,
    pub task_id: i64,
    pub platform: String,
    pub status: i16,
    pub result_url: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::generation_task::Entity",
        from = "Column::TaskId",
        to = "super::generation_task::Column::Id"
    )]
    Task,
}

impl Related<super::generation_task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

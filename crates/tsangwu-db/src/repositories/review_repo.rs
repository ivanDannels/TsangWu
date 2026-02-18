use sea_orm::*;
use sea_orm::sea_query::Expr;
use crate::entities::content_review;
use crate::DbConn;

pub struct ReviewRepo;

impl ReviewRepo {
    pub async fn find_by_task(db: &DbConn, task_id: i64) -> anyhow::Result<Option<content_review::Model>> {
        Ok(content_review::Entity::find()
            .filter(content_review::Column::TaskId.eq(task_id))
            .order_by_desc(content_review::Column::CreatedAt)
            .one(&db.conn)
            .await?)
    }

    pub async fn create(db: &DbConn, model: content_review::ActiveModel) -> anyhow::Result<content_review::Model> {
        Ok(model.insert(&db.conn).await?)
    }

    pub async fn update_status(db: &DbConn, id: i64, status: i16, result: Option<serde_json::Value>) -> anyhow::Result<()> {
        let mut update = content_review::Entity::update_many()
            .col_expr(content_review::Column::Status, Expr::value(status));
        if let Some(r) = result {
            update = update.col_expr(content_review::Column::Result, Expr::value(Some(r)));
        }
        update.filter(content_review::Column::Id.eq(id)).exec(&db.conn).await?;
        Ok(())
    }
}

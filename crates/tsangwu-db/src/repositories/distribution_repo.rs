use sea_orm::*;
use sea_orm::sea_query::Expr;
use crate::entities::distribution;
use crate::DbConn;

pub struct DistributionRepo;

impl DistributionRepo {
    pub async fn find_by_uid(db: &DbConn, uid: uuid::Uuid) -> anyhow::Result<Option<distribution::Model>> {
        Ok(distribution::Entity::find()
            .filter(distribution::Column::Uid.eq(uid))
            .one(&db.conn)
            .await?)
    }

    pub async fn list_by_task(db: &DbConn, task_id: i64) -> anyhow::Result<Vec<distribution::Model>> {
        Ok(distribution::Entity::find()
            .filter(distribution::Column::TaskId.eq(task_id))
            .order_by_desc(distribution::Column::CreatedAt)
            .all(&db.conn)
            .await?)
    }

    pub async fn create(db: &DbConn, model: distribution::ActiveModel) -> anyhow::Result<distribution::Model> {
        Ok(model.insert(&db.conn).await?)
    }

    pub async fn update_status(db: &DbConn, id: i64, status: i16, result_url: Option<String>) -> anyhow::Result<()> {
        let mut update = distribution::Entity::update_many()
            .col_expr(distribution::Column::Status, Expr::value(status));
        if let Some(url) = result_url {
            update = update.col_expr(distribution::Column::ResultUrl, Expr::value(Some(url)));
        }
        update.filter(distribution::Column::Id.eq(id)).exec(&db.conn).await?;
        Ok(())
    }
}

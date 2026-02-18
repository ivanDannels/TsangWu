use sea_orm::*;
use sea_orm::sea_query::Expr;
use crate::entities::copyright_record;
use crate::DbConn;

pub struct CopyrightRepo;

impl CopyrightRepo {
    pub async fn find_by_task(db: &DbConn, task_id: i64) -> anyhow::Result<Option<copyright_record::Model>> {
        Ok(copyright_record::Entity::find()
            .filter(copyright_record::Column::TaskId.eq(task_id))
            .order_by_desc(copyright_record::Column::CreatedAt)
            .one(&db.conn)
            .await?)
    }

    pub async fn create(db: &DbConn, model: copyright_record::ActiveModel) -> anyhow::Result<copyright_record::Model> {
        Ok(model.insert(&db.conn).await?)
    }

    pub async fn update_status(db: &DbConn, id: i64, status: i16) -> anyhow::Result<()> {
        copyright_record::Entity::update_many()
            .col_expr(copyright_record::Column::Status, Expr::value(status))
            .filter(copyright_record::Column::Id.eq(id))
            .exec(&db.conn)
            .await?;
        Ok(())
    }
}

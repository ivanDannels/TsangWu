use sea_orm::*;
use sea_orm::sea_query::Expr;
use crate::entities::generation_task;
use crate::DbConn;
use tsangwu_common::PageParams;

pub struct TaskRepo;

impl TaskRepo {
    pub async fn find_by_uid(db: &DbConn, uid: uuid::Uuid) -> anyhow::Result<Option<generation_task::Model>> {
        Ok(generation_task::Entity::find()
            .filter(generation_task::Column::Uid.eq(uid))
            .one(&db.conn)
            .await?)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> anyhow::Result<Option<generation_task::Model>> {
        Ok(generation_task::Entity::find_by_id(id).one(&db.conn).await?)
    }

    pub async fn list_by_project(db: &DbConn, project_id: i64, params: &PageParams) -> anyhow::Result<(Vec<generation_task::Model>, u64)> {
        let paginator = generation_task::Entity::find()
            .filter(generation_task::Column::ProjectId.eq(project_id))
            .order_by_desc(generation_task::Column::CreatedAt)
            .paginate(&db.conn, params.page_size);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(params.page.saturating_sub(1)).await?;
        Ok((items, total))
    }

    pub async fn list_by_user(db: &DbConn, user_id: i64, params: &PageParams) -> anyhow::Result<(Vec<generation_task::Model>, u64)> {
        let paginator = generation_task::Entity::find()
            .filter(generation_task::Column::UserId.eq(user_id))
            .order_by_desc(generation_task::Column::CreatedAt)
            .paginate(&db.conn, params.page_size);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(params.page.saturating_sub(1)).await?;
        Ok((items, total))
    }

    pub async fn create(db: &DbConn, model: generation_task::ActiveModel) -> anyhow::Result<generation_task::Model> {
        Ok(model.insert(&db.conn).await?)
    }

    pub async fn update_status(db: &DbConn, id: i64, status: i16) -> anyhow::Result<()> {
        generation_task::Entity::update_many()
            .col_expr(generation_task::Column::Status, Expr::value(status))
            .filter(generation_task::Column::Id.eq(id))
            .exec(&db.conn)
            .await?;
        Ok(())
    }

    pub async fn update_progress(db: &DbConn, id: i64, progress: i16) -> anyhow::Result<()> {
        generation_task::Entity::update_many()
            .col_expr(generation_task::Column::Progress, Expr::value(Some(progress)))
            .filter(generation_task::Column::Id.eq(id))
            .exec(&db.conn)
            .await?;
        Ok(())
    }
}

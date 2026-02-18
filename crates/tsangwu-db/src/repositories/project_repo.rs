use sea_orm::*;
use crate::entities::project;
use crate::DbConn;
use tsangwu_common::PageParams;

pub struct ProjectRepo;

impl ProjectRepo {
    pub async fn find_by_uid(db: &DbConn, uid: uuid::Uuid) -> anyhow::Result<Option<project::Model>> {
        Ok(project::Entity::find()
            .filter(project::Column::Uid.eq(uid))
            .one(&db.conn)
            .await?)
    }

    pub async fn list_by_owner(db: &DbConn, owner_id: i64, params: &PageParams) -> anyhow::Result<(Vec<project::Model>, u64)> {
        let paginator = project::Entity::find()
            .filter(project::Column::OwnerId.eq(owner_id))
            .order_by_desc(project::Column::CreatedAt)
            .paginate(&db.conn, params.page_size);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(params.page.saturating_sub(1)).await?;
        Ok((items, total))
    }

    pub async fn create(db: &DbConn, model: project::ActiveModel) -> anyhow::Result<project::Model> {
        Ok(model.insert(&db.conn).await?)
    }

    pub async fn update(db: &DbConn, model: project::ActiveModel) -> anyhow::Result<project::Model> {
        Ok(model.update(&db.conn).await?)
    }

    pub async fn delete(db: &DbConn, id: i64) -> anyhow::Result<()> {
        project::Entity::delete_by_id(id).exec(&db.conn).await?;
        Ok(())
    }
}

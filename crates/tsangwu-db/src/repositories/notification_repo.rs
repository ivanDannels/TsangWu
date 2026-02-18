use sea_orm::*;
use sea_orm::sea_query::Expr;
use crate::entities::notification;
use crate::DbConn;
use tsangwu_common::PageParams;

pub struct NotificationRepo;

impl NotificationRepo {
    pub async fn list_by_user(db: &DbConn, user_id: i64, params: &PageParams) -> anyhow::Result<(Vec<notification::Model>, u64)> {
        let paginator = notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id))
            .order_by_desc(notification::Column::CreatedAt)
            .paginate(&db.conn, params.page_size);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(params.page.saturating_sub(1)).await?;
        Ok((items, total))
    }

    pub async fn mark_read(db: &DbConn, id: i64) -> anyhow::Result<()> {
        notification::Entity::update_many()
            .col_expr(notification::Column::Read, Expr::value(true))
            .filter(notification::Column::Id.eq(id))
            .exec(&db.conn)
            .await?;
        Ok(())
    }

    pub async fn mark_all_read(db: &DbConn, user_id: i64) -> anyhow::Result<()> {
        notification::Entity::update_many()
            .col_expr(notification::Column::Read, Expr::value(true))
            .filter(notification::Column::UserId.eq(user_id))
            .filter(notification::Column::Read.eq(false))
            .exec(&db.conn)
            .await?;
        Ok(())
    }

    pub async fn create(db: &DbConn, model: notification::ActiveModel) -> anyhow::Result<notification::Model> {
        Ok(model.insert(&db.conn).await?)
    }

    pub async fn unread_count(db: &DbConn, user_id: i64) -> anyhow::Result<u64> {
        Ok(notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id))
            .filter(notification::Column::Read.eq(false))
            .count(&db.conn)
            .await?)
    }
}

use sea_orm::*;
use crate::entities::subscription;
use crate::DbConn;

pub struct SubscriptionRepo;

impl SubscriptionRepo {
    pub async fn find_active_by_user(db: &DbConn, user_id: i64) -> anyhow::Result<Option<subscription::Model>> {
        Ok(subscription::Entity::find()
            .filter(subscription::Column::UserId.eq(user_id))
            .filter(subscription::Column::Status.eq(1i16)) // 1 = active
            .one(&db.conn)
            .await?)
    }

    pub async fn create(db: &DbConn, model: subscription::ActiveModel) -> anyhow::Result<subscription::Model> {
        Ok(model.insert(&db.conn).await?)
    }

    pub async fn update(db: &DbConn, model: subscription::ActiveModel) -> anyhow::Result<subscription::Model> {
        Ok(model.update(&db.conn).await?)
    }
}

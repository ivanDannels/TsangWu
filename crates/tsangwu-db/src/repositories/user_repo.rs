use sea_orm::*;
use crate::entities::user;
use crate::DbConn;

pub struct UserRepo;

impl UserRepo {
    pub async fn find_by_uid(db: &DbConn, uid: uuid::Uuid) -> anyhow::Result<Option<user::Model>> {
        Ok(user::Entity::find()
            .filter(user::Column::Uid.eq(uid))
            .one(&db.conn)
            .await?)
    }

    pub async fn find_by_email(db: &DbConn, email: &str) -> anyhow::Result<Option<user::Model>> {
        Ok(user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(&db.conn)
            .await?)
    }

    pub async fn find_by_phone(db: &DbConn, phone: &str) -> anyhow::Result<Option<user::Model>> {
        Ok(user::Entity::find()
            .filter(user::Column::Phone.eq(phone))
            .one(&db.conn)
            .await?)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> anyhow::Result<Option<user::Model>> {
        Ok(user::Entity::find_by_id(id).one(&db.conn).await?)
    }

    pub async fn create(db: &DbConn, model: user::ActiveModel) -> anyhow::Result<user::Model> {
        Ok(model.insert(&db.conn).await?)
    }

    pub async fn update(db: &DbConn, model: user::ActiveModel) -> anyhow::Result<user::Model> {
        Ok(model.update(&db.conn).await?)
    }
}

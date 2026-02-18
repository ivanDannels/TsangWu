use sea_orm::*;
use crate::entities::{user_role, role, role_permission, permission};
use crate::DbConn;

pub struct RoleRepo;

impl RoleRepo {
    pub async fn find_role_codes_by_user(db: &DbConn, user_id: i64) -> anyhow::Result<Vec<String>> {
        #[derive(Debug, FromQueryResult)]
        struct RoleCode {
            code: String,
        }
        let roles = role::Entity::find()
            .select_only()
            .column(role::Column::Code)
            .inner_join(user_role::Entity)
            .filter(user_role::Column::UserId.eq(user_id))
            .into_model::<RoleCode>()
            .all(&db.conn)
            .await?;
        Ok(roles.into_iter().map(|r| r.code).collect())
    }

    pub async fn find_permissions_by_user(db: &DbConn, user_id: i64) -> anyhow::Result<Vec<(String, String)>> {
        #[derive(Debug, FromQueryResult)]
        struct PermRow {
            resource: String,
            action: String,
        }
        // permission -> role_permission -> user_role chain
        let perms = permission::Entity::find()
            .select_only()
            .column(permission::Column::Resource)
            .column(permission::Column::Action)
            .inner_join(role_permission::Entity)
            .join(
                JoinType::InnerJoin,
                role_permission::Relation::Role.def()
                    .rev()
                    .on_condition(|_left, _right| {
                        Condition::all()
                    }),
            )
            .join(
                JoinType::InnerJoin,
                role::Relation::UserRoles.def(),
            )
            .filter(user_role::Column::UserId.eq(user_id))
            .into_model::<PermRow>()
            .all(&db.conn)
            .await?;
        Ok(perms.into_iter().map(|p| (p.resource, p.action)).collect())
    }

    pub async fn has_permission(db: &DbConn, user_id: i64, resource: &str, action: &str) -> anyhow::Result<bool> {
        let perms = Self::find_permissions_by_user(db, user_id).await?;
        Ok(perms.iter().any(|(r, a)| r == resource && a == action))
    }
}

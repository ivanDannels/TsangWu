use sea_orm::*;
use crate::entities::usage_record;
use crate::DbConn;
use tsangwu_common::PageParams;

pub struct UsageRepo;

impl UsageRepo {
    pub async fn list_by_user_period(
        db: &DbConn,
        user_id: i64,
        from: chrono::DateTime<chrono::FixedOffset>,
        to: chrono::DateTime<chrono::FixedOffset>,
        params: &PageParams,
    ) -> anyhow::Result<(Vec<usage_record::Model>, u64)> {
        let paginator = usage_record::Entity::find()
            .filter(usage_record::Column::UserId.eq(user_id))
            .filter(usage_record::Column::CreatedAt.gte(from))
            .filter(usage_record::Column::CreatedAt.lte(to))
            .order_by_desc(usage_record::Column::CreatedAt)
            .paginate(&db.conn, params.page_size);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(params.page.saturating_sub(1)).await?;
        Ok((items, total))
    }

    pub async fn create(db: &DbConn, model: usage_record::ActiveModel) -> anyhow::Result<usage_record::Model> {
        Ok(model.insert(&db.conn).await?)
    }

    pub async fn sum_by_user_period(
        db: &DbConn,
        user_id: i64,
        from: chrono::DateTime<chrono::FixedOffset>,
        to: chrono::DateTime<chrono::FixedOffset>,
    ) -> anyhow::Result<i64> {
        #[derive(Debug, FromQueryResult)]
        struct SumResult {
            total: Option<i64>,
        }
        let result = usage_record::Entity::find()
            .select_only()
            .column_as(usage_record::Column::Amount.sum(), "total")
            .filter(usage_record::Column::UserId.eq(user_id))
            .filter(usage_record::Column::CreatedAt.gte(from))
            .filter(usage_record::Column::CreatedAt.lte(to))
            .into_model::<SumResult>()
            .one(&db.conn)
            .await?;
        Ok(result.and_then(|r| r.total).unwrap_or(0))
    }
}

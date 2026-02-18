use sea_orm::{sea_query::Expr, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Select};

/// 动态查询构建器
pub struct QueryBuilder<E: EntityTrait> {
    select: Select<E>,
    conditions: Condition,
}

impl<E: EntityTrait> QueryBuilder<E> {
    /// 创建新的查询构建器
    pub fn new() -> Self {
        Self {
            select: E::find(),
            conditions: Condition::all(),
        }
    }

    /// 添加等值条件
    pub fn eq<C: ColumnTrait>(mut self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.conditions = self.conditions.add(column.eq(value));
        self
    }

    /// 添加不等值条件
    pub fn ne<C: ColumnTrait>(mut self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.conditions = self.conditions.add(column.ne(value));
        self
    }

    /// 添加大于条件
    pub fn gt<C: ColumnTrait>(mut self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.conditions = self.conditions.add(column.gt(value));
        self
    }

    /// 添加大于等于条件
    pub fn gte<C: ColumnTrait>(mut self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.conditions = self.conditions.add(column.gte(value));
        self
    }

    /// 添加小于条件
    pub fn lt<C: ColumnTrait>(mut self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.conditions = self.conditions.add(column.lt(value));
        self
    }

    /// 添加小于等于条件
    pub fn lte<C: ColumnTrait>(mut self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.conditions = self.conditions.add(column.lte(value));
        self
    }

    /// 添加 LIKE 条件
    pub fn like<C: ColumnTrait>(mut self, column: C, pattern: &str) -> Self {
        self.conditions = self.conditions.add(column.like(pattern));
        self
    }

    /// 添加 IN 条件
    pub fn is_in<C: ColumnTrait, I>(mut self, column: C, values: I) -> Self
    where
        I: IntoIterator<Item = sea_orm::Value>,
    {
        self.conditions = self.conditions.add(column.is_in(values));
        self
    }

    /// 添加 IS NULL 条件
    pub fn is_null<C: ColumnTrait>(mut self, column: C) -> Self {
        self.conditions = self.conditions.add(column.is_null());
        self
    }

    /// 添加 IS NOT NULL 条件
    pub fn is_not_null<C: ColumnTrait>(mut self, column: C) -> Self {
        self.conditions = self.conditions.add(column.is_not_null());
        self
    }

    /// 添加 BETWEEN 条件
    pub fn between<C: ColumnTrait, V: Into<sea_orm::Value>>(
        mut self,
        column: C,
        min: V,
        max: V,
    ) -> Self {
        self.conditions = self.conditions.add(column.between(min, max));
        self
    }

    /// 添加可选条件（如果值为 Some 则添加）
    pub fn eq_opt<C: ColumnTrait>(self, column: C, value: Option<impl Into<sea_orm::Value>>) -> Self {
        if let Some(v) = value {
            self.eq(column, v)
        } else {
            self
        }
    }

    /// 添加可选 LIKE 条件
    pub fn like_opt<C: ColumnTrait>(self, column: C, pattern: Option<&str>) -> Self {
        if let Some(p) = pattern {
            self.like(column, &format!("%{}%", p))
        } else {
            self
        }
    }

    /// 升序排序
    pub fn order_asc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.select = self.select.order_by_asc(column);
        self
    }

    /// 降序排序
    pub fn order_desc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.select = self.select.order_by_desc(column);
        self
    }

    /// 限制结果数量
    pub fn limit(mut self, limit: u64) -> Self {
        self.select = self.select.limit(limit);
        self
    }

    /// 偏移量
    pub fn offset(mut self, offset: u64) -> Self {
        self.select = self.select.offset(offset);
        self
    }

    /// 构建最终查询
    pub fn build(mut self) -> Select<E> {
        self.select.filter(self.conditions)
    }
}

impl<E: EntityTrait> Default for QueryBuilder<E> {
    fn default() -> Self {
        Self::new()
    }
}

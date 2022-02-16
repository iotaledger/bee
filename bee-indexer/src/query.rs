// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::status;
use crate::types::FilterOptions;

use sea_orm as orm;
use sea_orm::{DatabaseBackend, Statement};
use sea_query::{Alias, Cond, Expr, JoinType, Order};

pub(crate) trait OutputTable
where
    Self: orm::EntityTrait,
{
    type FilterOptions: Into<sea_query::Cond>;
    fn created_at_col() -> <Self as orm::EntityTrait>::Column;
    fn output_id_col() -> <Self as orm::EntityTrait>::Column;
}

#[must_use]
pub(crate) struct QueryBuilder<T: OutputTable> {
    entity: T,
    options: FilterOptions<T>,
}

impl<T: OutputTable> QueryBuilder<T> {
    pub(crate) fn new(entity: T, options: FilterOptions<T>) -> Self {
        Self { entity, options }
    }

    pub(crate) fn build(self, backend: DatabaseBackend) -> Statement {
        let mut stmt = sea_query::Query::select();

        stmt.from(self.entity);

        let alias_cursor = Alias::new("cursor");

        stmt.column(T::output_id_col())
            .expr_as(
                Expr::cust("printf('%08X', `created_at`) || hex(output_id)"), // TODO Use sea-query
                alias_cursor.clone(),
            )
            .column(status::Column::CurrentMilestoneIndex) // TODO: Remove if everything works
            .join(JoinType::InnerJoin, status::Entity, Cond::any())
            .order_by_columns(vec![
                (T::created_at_col(), Order::Asc),
                (T::output_id_col(), Order::Asc),
            ]);

        if self.options.pagination.page_size > 0 {
            if let Some(cursor) = self.options.pagination.cursor.clone() {
                // TODO: write testcase
                stmt.cond_where(Expr::col(alias_cursor).gte(cursor.to_uppercase()));
            }
            stmt.limit(self.options.pagination.page_size + 1);
        }

        let condition: sea_query::Cond = self.options.into();
        stmt.cond_where(condition);

        backend.build(&stmt)
    }
}

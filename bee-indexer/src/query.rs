// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use crate::{status, UniversalFilterOptions};

use sea_orm::{ColumnTrait, EntityTrait, DatabaseBackend, Statement, Condition};
use sea_query::{Alias, Cond, Expr, JoinType, Order};

pub(crate) trait OutputFilterOptions<Entity: EntityTrait, Column: ColumnTrait> {
    fn entity() -> Entity;
    fn created_at_col() -> Column;
    fn output_id_col() -> Column;
    fn filter(self) -> Condition;
}

pub(crate) struct QueryBuilder<E, C, F>
where
    E: EntityTrait,
    C: ColumnTrait,
    F: OutputFilterOptions<E, C>,
{
    universal_options: UniversalFilterOptions,
    output_options: F,
    _phantom_e: std::marker::PhantomData<E>,
    _phantom_c: std::marker::PhantomData<C>,
}

impl<E, C, F> QueryBuilder<E, C, F>
where
    E: EntityTrait,
    C: ColumnTrait,
    F: OutputFilterOptions<E, C>,
{

    pub(crate) fn new(universal_options: UniversalFilterOptions, output_options: F) -> Self {
        Self {
            universal_options, output_options, _phantom_e: PhantomData, _phantom_c: PhantomData,
        }
    }

    pub(crate) fn build(self, backend: DatabaseBackend) -> Statement {
        let mut stmt = sea_query::Query::select();

        stmt.from(F::entity());

        let alias_cursor = Alias::new("cursor");

        stmt.column(F::output_id_col())
            .expr_as(
                Expr::cust("printf('%08X', `created_at`) || hex(output_id)"), // TODO Use sea-query
                alias_cursor.clone(),
            )
            .column(status::Column::CurrentMilestoneIndex) // TODO: Remove if everything works
            .join(JoinType::InnerJoin, status::Entity, Cond::any())
            .order_by_columns(vec![
                (F::created_at_col(), Order::Asc),
                (F::output_id_col(), Order::Asc),
            ]);

        if self.universal_options.pagination.page_size > 0 {
            if let Some(cursor) = self.universal_options.pagination.cursor {
                // TODO: write testcase
                stmt.cond_where(Expr::col(alias_cursor).gte(cursor.to_uppercase()));
            }
            stmt.limit(self.universal_options.pagination.page_size + 1);
        }

        let mut filter = OutputFilterOptions::filter(self.output_options);

        if let Some(timestamp) = self.universal_options.created_before {
            filter = filter.add(F::created_at_col().lt(timestamp));
        }
        if let Some(timestamp) = self.universal_options.created_after {
            filter = filter.add(F::created_at_col().gt(timestamp));
        }

        stmt.cond_where(filter);

        backend.build(&stmt)
    }
}

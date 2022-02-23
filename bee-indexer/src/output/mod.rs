pub(crate) mod alias;
pub(crate) mod basic;
pub(crate) mod foundry;
pub(crate) mod nft;

use sea_orm as orm;
use sea_orm::{ColumnTrait, DatabaseBackend, Statement};
use sea_query::{Alias, Cond, Expr, JoinType, Order};

use crate::{
    status,
    types::{AddressDb, Pagination, TimestampFilterOptions},
    Error,
};

use bee_message::address::Address;

use packable::PackableExt;

#[inline(always)]
pub(crate) fn address_dto_option_packed(
    option: Option<String>,
    err_str: &'static str,
) -> Result<Option<AddressDb>, Error> {
    Ok(option
        .map(|addr| {
            Address::try_from_bech32(&addr)
                .map_err(|_| Error::InvalidField(err_str))
                .map(|addr| addr.pack_to_vec())
        })
        .transpose()?)
}

pub(crate) trait OutputTable
where
    Self: orm::EntityTrait,
{
    type FilterOptions: Into<sea_query::Cond>;
    type FilterOptionsDto: TryInto<Self::FilterOptions, Error = Error>;

    const ENTITY: Self;

    fn created_at_col() -> <Self as orm::EntityTrait>::Column;
    fn output_id_col() -> <Self as orm::EntityTrait>::Column;

    fn filter_statement<F: Into<sea_query::Cond>>(
        backend: DatabaseBackend,
        pagination: Pagination,
        timestamp: TimestampFilterOptions,
        options: F,
    ) -> Statement {
        let alias_cursor = Alias::new("cursor");
        let mut stmt = sea_query::Query::select();

        stmt.from(Self::ENTITY)
            .column(Self::output_id_col())
            .expr_as(
                Expr::cust("printf('%08X', `created_at`) || hex(output_id)"), // TODO Use sea-query
                alias_cursor.clone(),
            )
            .column(status::Column::CurrentMilestoneIndex) // TODO: Remove if everything works
            .join(JoinType::InnerJoin, status::Entity, Cond::any())
            .order_by_columns(vec![
                (Self::created_at_col(), Order::Asc),
                (Self::output_id_col(), Order::Asc),
            ]);

        if pagination.page_size > 0 {
            if let Some(cursor) = pagination.cursor.clone() {
                // TODO: write testcase
                stmt.cond_where(Expr::col(alias_cursor).gte(cursor.to_uppercase()));
            }
            stmt.limit(pagination.page_size + 1);
        }

        let condition = Cond::all()
            .add_option(
                timestamp
                    .created_after
                    .map(|timestamp| Self::created_at_col().gt(timestamp)),
            )
            .add_option(
                timestamp
                    .created_before
                    .map(|timestamp| Self::created_at_col().lt(timestamp)),
            )
            .add(options.into());
        stmt.cond_where(condition);

        backend.build(&stmt)
    }
}

pub(crate) trait IndexedOutputTable: OutputTable {
    fn id_col() -> <Self as orm::EntityTrait>::Column;

    fn get_id_statement(backend: DatabaseBackend, id: Vec<u8>) -> Statement {
        let mut query = sea_query::Query::select();
        let statement = query
            .from(Self::ENTITY)
            .column(Self::output_id_col())
            .cond_where(Self::id_col().eq(id));
        backend.build(statement)
    }
}

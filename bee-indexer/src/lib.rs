// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod error;
pub(crate) mod output;
pub(crate) mod query;
pub(crate) mod status;
pub(crate) mod types;

// TODO: Check `pub(crate)` visibility of all types.

pub use error::Error;

pub(crate) use query::QueryBuilder;
use query::{IndexedOutputTable, OutputTable};

pub use types::dtos::{AddressDto, AliasFilterOptionsDto, FoundryFilterOptionsDto};

use output::{alias, basic, foundry, nft};

use bee_ledger::workers::event::{OutputConsumed, OutputCreated};
use bee_message::{milestone::MilestoneIndex, output::Output};

use packable::PackableExt;

use sea_orm::{
    prelude::*, ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, FromQueryResult, NotSet,
    QuerySelect, Schema, Set,
};

use types::{
    dtos::{BasicFilterOptionsDto, NftFilterOptionsDto},
    responses::OutputsResponse,
    AddressDb, FilterOptions, MilestoneIndexDb,
};

pub struct Indexer {
    db: DatabaseConnection,
}

impl Indexer {
    pub async fn new() -> Result<Self, Error> {
        // For now, the database lives in memory.
        let db = Database::connect("sqlite::memory:").await.unwrap();

        let builder = db.get_database_backend();
        let schema = Schema::new(builder);

        db.execute(builder.build(&schema.create_table_from_entity(alias::Entity)))
            .await
            .map_err(Error::DatabaseError)?;
        db.execute(builder.build(&schema.create_table_from_entity(basic::Entity)))
            .await
            .map_err(Error::DatabaseError)?;
        db.execute(builder.build(&schema.create_table_from_entity(foundry::Entity)))
            .await
            .map_err(Error::DatabaseError)?;
        db.execute(builder.build(&schema.create_table_from_entity(nft::Entity)))
            .await
            .map_err(Error::DatabaseError)?;

        db.execute(builder.build(&schema.create_table_from_entity(status::Entity)))
            .await
            .map_err(Error::DatabaseError)?;

        // TODO: Create indices!

        // Initialize the status table.
        let status = status::ActiveModel {
            id: Set(1),
            current_milestone_index: Set(0),
        };
        status.insert(&db).await.map_err(Error::DatabaseError)?;

        Ok(Self { db })
    }

    pub async fn update_status(&self, milestone_index: MilestoneIndex) -> Result<(), Error> {
        let status = status::Entity::find_by_id(1)
            .one(&self.db)
            .await
            .map_err(Error::DatabaseError)?;
        // Safety: There is always only one status at `id = 1`.
        let mut status: status::ActiveModel = status.unwrap().into();
        status.current_milestone_index = Set(milestone_index.0);
        // We are not interested int the returned `id`.
        let _ = status.update(&self.db).await.map_err(Error::DatabaseError)?;
        Ok(())
    }

    pub async fn current_status(&self) -> Result<MilestoneIndex, Error> {
        let status = status::Entity::find_by_id(1)
            .one(&self.db)
            .await
            .map_err(Error::DatabaseError)?;
        // Safety: We can unwrap, because we guarantee that there is always one row in the table.
        Ok(MilestoneIndex(status.unwrap().current_milestone_index))
    }

    pub async fn process_created_output(&self, created: &OutputCreated) -> Result<(), Error> {
        match created.output.inner() {
            Output::Alias(output) => {
                let alias = alias::ActiveModel {
                    alias_id: Set(output.alias_id().pack_to_vec()),
                    output_id: Set(created.output_id.pack_to_vec()),
                    created_at: Set(created.output.milestone_timestamp()),
                    amount: Set(output.amount() as i64),
                    state_controller: Set(output.state_controller().pack_to_vec()),
                    governor: Set(output.governor().pack_to_vec()),
                    issuer: NotSet, // TODO: Fix
                    sender: NotSet, // TODO: Fix
                };
                alias.insert(&self.db).await.map_err(Error::DatabaseError)?;
            }
            _ => todo!(),
        }

        Ok(())
    }

    pub async fn process_spent_output(&self, consumed: &OutputConsumed) -> Result<(), sea_orm::error::DbErr> {
        match &consumed.output {
            Output::Alias(output) => {
                let alias = alias::Entity::find_by_id(output.alias_id().pack_to_vec())
                    .one(&self.db)
                    .await?;
                if let Some(alias) = alias {
                    alias.delete(&self.db).await?;
                }
            }
            _ => todo!(),
        }

        Ok(())
    }

    

    pub(crate) async fn get_id<T: IndexedOutputTable>(&self, table: T, id: String) -> Result<Option<String>, Error> {
        let id_bytes = hex::decode(id).map_err(|_| Error::InvalidId)?;
        
        let mut query = sea_query::Query::select();
        let statement = query.from(table).column(T::output_id_col()).cond_where(T::id_col().eq(id_bytes));
        let stmt = self.db.get_database_backend().build(statement);
        // TODO: Sanitize (check for sql injections).
        let query_result = IdResult::find_by_statement(stmt).one(&self.db).await
        .map_err(Error::DatabaseError)?;
            // .select_only()
            // .column(T::id_col())
            // .filter(T::id_col().eq(id_bytes))
            // .one(&self.db).await.map_err(Error::DatabaseError)?;

            // query_result.map(|r| r.output_id)
            Ok(query_result.map(|r| hex::encode(r.output_id)))
    }

    pub(crate) async fn outputs_with_filters<T: OutputTable>(
        &self,
        table: T,
        options_dto: impl TryInto<FilterOptions<T>, Error = Error>,
    ) -> Result<OutputsResponse, Error> {
        let options: FilterOptions<T> = options_dto.try_into()?;
        let page_size = options.pagination.page_size;

        let statement = QueryBuilder::new(table, options).build(self.db.get_database_backend());

        let query_results = JoinedResult::find_by_statement(statement)
            .all(&self.db)
            .await
            .map_err(Error::DatabaseError)?;

        let mut response = OutputsResponse {
            items: query_results
                .iter()
                .map(|r| hex::encode(r.output_id.clone())) // TODO: Get rid of clone
                .collect(),
            ledger_index: query_results
                .first()
                .map(|r| r.current_milestone_index)
                .unwrap_or(0)
                .into(),
            cursor: None,
        };

        if page_size > 0 && query_results.len() > page_size as usize {
            // We have queried one element to many to get the cursor for the next page.
            response.cursor = Some(query_results.last().unwrap().cursor.clone().to_lowercase());
            response.items.pop();
        }

        Ok(response)
    }

    // TODO: Make generic (or use macro)
    pub async fn alias_outputs_with_filters(
        &self,
        options_dto: AliasFilterOptionsDto,
    ) -> Result<OutputsResponse, Error> {
        self.outputs_with_filters(alias::Entity, options_dto).await
    }

    // TODO: Make generic (or use macro)
    pub async fn basic_outputs_with_filters(
        &self,
        options_dto: BasicFilterOptionsDto,
    ) -> Result<OutputsResponse, Error> {
        self.outputs_with_filters(basic::Entity, options_dto).await
    }

    // TODO: Make generic (or use macro)
    pub async fn foundry_outputs_with_filters(
        &self,
        options_dto: FoundryFilterOptionsDto,
    ) -> Result<OutputsResponse, Error> {
        self.outputs_with_filters(foundry::Entity, options_dto).await
    }

    pub async fn get_output_id_for_alias_id(&self, id: String) -> Result<Option<String>, Error> {
        self.get_id(alias::Entity, id).await
    }

    // TODO: Make generic (or use macro)
    pub async fn nft_outputs_with_filters(&self, options_dto: NftFilterOptionsDto) -> Result<OutputsResponse, Error> {
        self.outputs_with_filters(nft::Entity, options_dto).await
    }
}

#[derive(Debug, FromQueryResult)]
struct IdResult {
    output_id: Vec<u8>,
}

#[derive(Debug, FromQueryResult)]
pub struct JoinedResult {
    pub output_id: AddressDb,
    pub current_milestone_index: MilestoneIndexDb,
    pub cursor: String,
}

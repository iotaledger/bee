// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod error;
pub(crate) mod output;
pub(crate) mod query;
pub(crate) mod status;
pub(crate) mod types;

// TODO: Check `pub(crate)` visibility of all types.

pub use error::Error;

use query::OutputTable;
pub(crate) use query::QueryBuilder;

pub use types::dtos::{AddressDto, AliasFilterOptionsDto, FoundryFilterOptionsDto};

use output::alias;

use bee_ledger::workers::event::{OutputConsumed, OutputCreated};
use bee_message::{milestone::MilestoneIndex, output::Output};

use packable::PackableExt;

use sea_orm::{
    prelude::*, ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, FromQueryResult, NotSet,
    Schema, Set,
};

use types::{responses::OutputsResponse, AddressDb, MilestoneIndexDb, FilterOptions};

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
                    amount: Set(output.amount().to_le_bytes().to_vec()),
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

    // TODO: Use `OutputConsumed`
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

    //     pub async fn foundry_outputs_with_filters(
    //         &self,
    //         options_dto: FoundryFilterOptionsDto,
    //     ) -> Result<IndexedOutputs, Error> {
    //         let pagination: Pagination = options_dto.universal.pagination.try_into()?;
    //         let page_size = pagination.page_size;

    //         let statement = QueryBuilder::new(pagination).build(self.db.get_database_backend(), foundry::Entity);

    //         let query_results = JoinedResult::find_by_statement(statement)
    //             .all(&self.db)
    //             .await
    //             .map_err(Error::DatabaseError)?;

    //         let mut result = IndexedOutputs {
    //             output_ids: query_results
    //                 .iter()
    //                 .map(|r| {
    //                     let bytes: [u8; OutputId::LENGTH] = r.output_id.clone().try_into().unwrap();
    //                     bytes.try_into().unwrap()
    //                 })
    //                 .collect(),
    //             cursors: query_results.iter().map(|r| r.cursor.clone().to_lowercase()).collect(),
    //             milestone_index: query_results
    //                 .first()
    //                 .map(|r| r.current_milestone_index)
    //                 .unwrap_or(0)
    //                 .into(),
    //             page_size,
    //             cursor: None,
    //         };

    //         if page_size > 0 && query_results.len() > page_size as usize {
    //             // We have queried one element to many to get the cursor for the next page.
    //             result.cursor = Some(query_results.last().unwrap().cursor.clone().to_lowercase());
    //             result.output_ids.pop();
    //         }

    //         Ok(result)
    //     }
}

#[derive(Debug, FromQueryResult)]
pub struct JoinedResult {
    pub output_id: AddressDb,
    pub current_milestone_index: MilestoneIndexDb,
    pub cursor: String,
}

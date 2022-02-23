// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod error;
pub(crate) mod output;
pub(crate) mod status;
pub(crate) mod types;

// TODO: Check `pub(crate)` visibility of all types.

pub use error::Error;

use output::{IndexedOutputTable, OutputTable};

use sqlx::SqlitePool;
pub use types::dtos::{AddressDto, AliasFilterOptionsDto, FoundryFilterOptionsDto, FilterOptionsDto};

use output::{alias, basic, foundry, nft};

use bee_ledger::workers::event::{OutputConsumed, OutputCreated};
use bee_message::{address::Address, milestone::MilestoneIndex, output::Output};

use packable::PackableExt;

use sea_orm::{
    prelude::*, ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait, FromQueryResult, NotSet, Schema,
    Set, SqlxSqliteConnector,
};

use types::{
    dtos::{BasicFilterOptionsDto, NftFilterOptionsDto},
    responses::OutputsResponse,
    AddressDb, MilestoneIndexDb, Pagination,
};

pub struct Indexer {
    db: DatabaseConnection,
}

impl Indexer {
    pub async fn new_in_memory() -> Result<Self, Error> {
        Self::new("sqlite::memory:").await
    }

    pub async fn new(location: &str) -> Result<Self, Error> {
        // `sea_orm` won't automatically create a database if non exists yet.
        // To ship around this we use `sqlx` directly here.
        let pool = SqlitePool::connect(location)
            .await
            .map_err(Error::DatabaseConnectionError)?;
        let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);

        let builder = db.get_database_backend();
        let schema = Schema::new(builder);

        // `alias_output` table
        db.execute(builder.build(&schema.create_table_from_entity(alias::Entity)))
            .await
            .map_err(Error::DatabaseError)?;
        let statement = sea_query::Index::create().name("alias_state_controller").table(alias::Entity).col(alias::Column::StateController).to_owned();
        db.execute(builder.build(&statement)).await.map_err(Error::DatabaseError)?;
        let statement = sea_query::Index::create().name("alias_governor").table(alias::Entity).col(alias::Column::Governor).to_owned();
        db.execute(builder.build(&statement)).await.map_err(Error::DatabaseError)?;
        let statement = sea_query::Index::create().name("alias_issuer").table(alias::Entity).col(alias::Column::Issuer).to_owned();
        db.execute(builder.build(&statement)).await.map_err(Error::DatabaseError)?;
        let statement = sea_query::Index::create().name("alias_sender").table(alias::Entity).col(alias::Column::Sender).to_owned();
        db.execute(builder.build(&statement)).await.map_err(Error::DatabaseError)?;
        

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

    pub(crate) async fn get_id<T>(&self, id: String) -> Result<Option<String>, Error> where T: IndexedOutputTable {
        let id_bytes = hex::decode(&id)
            .map_err(|_| Error::InvalidId)?;

        let statement = T::get_id_statement(self.db.get_database_backend(), id_bytes);
        
        // TODO: Dow we need sanitize (check for sql injections)?
        let query_result = IdResult::find_by_statement(statement)
            .one(&self.db)
            .await
            .map_err(Error::DatabaseError)?;

        Ok(query_result.map(|r| hex::encode(r.output_id)))
    }

    pub(crate) async fn outputs_with_filters<T>(
        &self,
        options_dto: FilterOptionsDto<T::FilterOptionsDto>,
    ) -> Result<OutputsResponse, Error>
    where
        T: OutputTable,
    {
        let output_options: T::FilterOptions = options_dto.inner.try_into()?;
        let timestamp = options_dto.timestamp.try_into()?;
        let pagination: Pagination = options_dto.pagination.try_into()?;
        let page_size = pagination.page_size;

        let statement = T::filter_statement(self.db.get_database_backend(), pagination, timestamp, output_options);

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
        options_dto: FilterOptionsDto<AliasFilterOptionsDto>,
    ) -> Result<OutputsResponse, Error> {
        self.outputs_with_filters::<alias::Entity>(options_dto).await
    }

    // // TODO: Make generic (or use macro)
    // pub async fn basic_outputs_with_filters(
    //     &self,
    //     options_dto: BasicFilterOptionsDto,
    // ) -> Result<OutputsResponse, Error> {
    //     self.outputs_with_filters(basic::Entity, options_dto).await
    // }

    // // TODO: Make generic (or use macro)
    // pub async fn foundry_outputs_with_filters(
    //     &self,
    //     options_dto: FoundryFilterOptionsDto,
    // ) -> Result<OutputsResponse, Error> {
    //     self.outputs_with_filters(foundry::Entity, options_dto).await
    // }

    pub async fn get_output_id_for_alias_id(&self, id: String) -> Result<Option<String>, Error> {
        self.get_id::<alias::Entity>(id).await
    }

    // // TODO: Make generic (or use macro)
    // pub async fn nft_outputs_with_filters(&self, options_dto: NftFilterOptionsDto) -> Result<OutputsResponse, Error> {
    //     self.outputs_with_filters(nft::Entity, options_dto).await
    // }
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

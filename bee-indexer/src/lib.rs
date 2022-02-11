// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod alias;
pub(crate) mod error;
pub(crate) mod extended;
pub(crate) mod foundry;
pub(crate) mod nft;
pub(crate) mod status;
pub(crate) mod types;

// TODO: Check `pub(crate)` visibility of all types.

use std::time::SystemTime;

use bee_ledger::workers::event::{OutputConsumed, OutputCreated};
use bee_message::{
    address::Address,
    milestone::MilestoneIndex,
    output::{Output, OutputId},
};

use packable::PackableExt;

use sea_orm::{
    prelude::*, ActiveModelTrait, Condition, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
    FromQueryResult, JoinType, NotSet, Order, Schema, Set,
};

use sea_query::{Alias, Cond, Expr};
use types::{AddressDb, MilestoneIndexDb};

pub use error::IndexerError;

pub struct Indexer {
    db: DatabaseConnection,
}

pub const HEX_CURSOR_LENGTH: usize = (std::mem::size_of::<MilestoneIndex>() + OutputId::LENGTH) * 2;

impl Indexer {
    pub async fn new() -> Result<Self, IndexerError> {
        // For now, the database lives in memory.
        let db = Database::connect("sqlite::memory:").await.unwrap();

        let builder = db.get_database_backend();
        let schema = Schema::new(builder);

        db.execute(builder.build(&schema.create_table_from_entity(alias::Entity)))
            .await
            .map_err(IndexerError::DatabaseError)?;
        db.execute(builder.build(&schema.create_table_from_entity(status::Entity)))
            .await
            .map_err(IndexerError::DatabaseError)?;

        // TODO: Create indices!

        // Initialize the status table.
        let status = status::ActiveModel {
            id: Set(1),
            current_milestone_index: Set(0),
        };
        status.insert(&db).await.map_err(IndexerError::DatabaseError)?;

        Ok(Self { db })
    }

    pub async fn update_status(&self, milestone_index: MilestoneIndex) -> Result<(), IndexerError> {
        let status = status::Entity::find_by_id(1)
            .one(&self.db)
            .await
            .map_err(IndexerError::DatabaseError)?;
        // Safety: There is always only one status at `id = 1`.
        let mut status: status::ActiveModel = status.unwrap().into();
        status.current_milestone_index = Set(milestone_index.0);
        // We are not interested int the returned `id`.
        let _ = status.update(&self.db).await.map_err(IndexerError::DatabaseError)?;
        Ok(())
    }

    pub async fn current_status(&self) -> Result<MilestoneIndex, IndexerError> {
        let status = status::Entity::find_by_id(1)
            .one(&self.db)
            .await
            .map_err(IndexerError::DatabaseError)?;
        // Safety: We can unwrap, because we guarantee that there is always one row in the table.
        Ok(MilestoneIndex(status.unwrap().current_milestone_index))
    }

    pub async fn process_created_output(&self, created: &OutputCreated) -> Result<(), IndexerError> {
        let output_id = created.output_id;

        match &created.output {
            Output::Alias(output) => {
                let alias = alias::ActiveModel {
                    // TODO: Use binary and let sqlite do the hex stuff
                    alias_id: Set(hex::encode(output.alias_id())),
                    output_id: Set(output_id.pack_to_vec()),
                    created_at: Set(SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as u32), // TODO: Get from output
                    amount: Set(output.amount() as i64), // TODO: Fix type?
                    state_controller: Set(output.state_controller().pack_to_vec()),
                    governor: Set(output.governor().pack_to_vec()),
                    issuer: NotSet, // TODO: Fix
                    sender: NotSet, // TODO: Fix
                };
                alias.insert(&self.db).await.map_err(IndexerError::DatabaseError)?;
            }
            _ => todo!(),
        }

        Ok(())
    }

    pub async fn process_spent_output(&self, consumed: &OutputConsumed) -> Result<(), sea_orm::error::DbErr> {
        match &consumed.output {
            Output::Alias(output) => {
                let alias = alias::Entity::find_by_id(hex::encode(output.alias_id()))
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

    pub async fn alias_outputs_with_filters(
        &self,
        options: AliasFilterOptions,
    ) -> Result<IndexedOutputs, IndexerError> {
        let builder = self.db.get_database_backend();

        let mut stmt = sea_query::Query::select();

        let cursor_alias = Alias::new("cursor");

        stmt.column(alias::Column::OutputId)
            .expr_as(
                Expr::cust("printf('%08X', `created_at`) || hex(output_id)"),
                cursor_alias.clone(),
            )
            .column(status::Column::CurrentMilestoneIndex) // TODO: Remove if everything works
            .from(alias::Entity)
            .join(JoinType::InnerJoin, status::Entity, Cond::any())
            .order_by_columns(vec![
                (alias::Column::CreatedAt, Order::Asc),
                (alias::Column::OutputId, Order::Asc),
            ]);

        let mut filter = Condition::all();

        if options.page_size > 0 {
            if let Some(cursor) = options.cursor {
                if cursor.len() != HEX_CURSOR_LENGTH {
                    return Err(IndexerError::InvalidCursorLength(cursor.len()));
                } else {
                    filter = filter.add(Expr::col(cursor_alias).gte(cursor.to_uppercase()));
                }
            }
            stmt.limit(options.page_size + 1);
        }

        if let Some(state_controller) = options.state_controller {
            filter = filter.add(alias::Column::StateController.eq(state_controller.pack_to_vec()));
        }
        if let Some(governor) = options.governor {
            filter = filter.add(alias::Column::Governor.eq(governor.pack_to_vec()));
        }
        if let Some(sender) = options.sender {
            filter = filter.add(alias::Column::Sender.eq(sender.pack_to_vec()));
        }
        if let Some(issuer) = options.issuer {
            filter = filter.add(alias::Column::Issuer.eq(issuer.pack_to_vec()));
        }

        stmt.cond_where(filter);

        let query_results = JoinedResult::find_by_statement(builder.build(&stmt))
            .all(&self.db)
            .await
            .map_err(IndexerError::DatabaseError)?;

        let mut result = IndexedOutputs {
            output_ids: query_results
                .iter()
                .map(|r| {
                    let bytes: [u8; OutputId::LENGTH] = r.output_id.clone().try_into().unwrap();
                    bytes.try_into().unwrap()
                })
                .collect(),
            cursors: query_results.iter().map(|r| r.cursor.clone().to_lowercase()).collect(),
            milestone_index: query_results
                .first()
                .map(|r| r.current_milestone_index)
                .unwrap_or(0)
                .into(),
            page_size: options.page_size,
            cursor: None,
        };

        if options.page_size > 0 && query_results.len() > options.page_size as usize {
            // We have queried one element to many to get the cursor for the next page.
            result.cursor = Some(query_results.last().unwrap().cursor.clone().to_lowercase());
            result.output_ids.pop(); 
        }

        Ok(result)
    }
}

// TODO: Consider creating a builder for this.
#[derive(Debug, Default)]
pub struct AliasFilterOptions {
    pub state_controller: Option<Address>,
    pub governor: Option<Address>,
    pub issuer: Option<Address>,
    pub sender: Option<Address>,
    pub page_size: u64, // TODO: We should settle on a sensible default value (from config, maybe)?
    pub cursor: Option<String>,
}

#[derive(Debug, FromQueryResult)]
pub struct JoinedResult {
    pub output_id: AddressDb,
    pub current_milestone_index: MilestoneIndexDb,
    pub cursor: String,
}

#[derive(Debug)]
pub struct IndexedOutputs {
    pub output_ids: Vec<OutputId>,
    pub cursors: Vec<String>, // TODO: Remove
    pub milestone_index: MilestoneIndex,
    pub page_size: u64,
    pub cursor: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_offset_length() {
        assert_eq!(HEX_CURSOR_LENGTH, 76);
    }

    // TODO: Testcase for max amount transaction
}

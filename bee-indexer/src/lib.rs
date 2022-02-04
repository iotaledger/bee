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

use chrono::Local;

use bee_ledger::workers::event::{OutputConsumed, OutputCreated};
use bee_message::{
    address::Address,
    milestone::MilestoneIndex,
    output::{Output, OutputId},
};

use packable::PackableExt;

use sea_orm::{
    prelude::*, ActiveModelTrait, Condition, ConnectionTrait, Database, DatabaseConnection,
    EntityTrait, FromQueryResult, JoinType, NotSet, Order, Schema, Set,
};

use chrono::NaiveDateTime;

use tokio_stream::StreamExt;

use sea_query::{Cond, Expr, Alias};
use types::{AddressDb, MilestoneIndexDb};

pub use error::IndexerError;

pub struct Indexer {
    db: DatabaseConnection,
}

pub const OFFSET_LENGTH: usize = std::mem::size_of::<MilestoneIndex>() + OutputId::LENGTH;

#[inline(always)]
fn offset_to_naive_date_time(offset: &[u8]) -> Result<NaiveDateTime, IndexerError> {
    let bytes = (&offset[0..4]).try_into().map_err(IndexerError::OffsetParseError)?;
    let timestamp_secs = u32::from_le_bytes(bytes) as i64;
    Ok(NaiveDateTime::from_timestamp(timestamp_secs, 0))
}

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
                    created_at: Set(Local::now().naive_local()),
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
        //) -> Result<Vec<(OutputResult, Option<MilestoneResult>)>, IndexerError> {
        let builder = self.db.get_database_backend();

        let mut stmt = sea_query::Query::select();

        stmt.column(alias::Column::OutputId)
            .expr_as(Expr::cust("printf('%08X', strftime('%s', `created_at`)) || hex(output_id)"), Alias::new("cursor"))
            .column(status::Column::CurrentMilestoneIndex)
            .from(alias::Entity)
            .join(JoinType::InnerJoin, status::Entity, Cond::any())
            .order_by(alias::Column::CreatedAt, Order::Asc)
            .order_by(alias::Column::OutputId, Order::Asc);

        let mut filter = Condition::all();

        if options.page_size > 0 {
            if let Some(offset) = options.offset {
                if offset.len() != OFFSET_LENGTH {
                    return Err(IndexerError::InvalidOffsetLength(offset.len()));
                } else {
                    let (timestamp, output_id) = offset.split_at(MilestoneIndex::LENGTH);
                    let created_at = offset_to_naive_date_time(&timestamp)?;
                    filter = filter.add(alias::Column::CreatedAt.gte(created_at));
                    filter = filter.add(alias::Column::OutputId.gte(hex::encode(output_id)));
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

        // TODO: Pagination

        let mut stream = JoinedResult::find_by_statement(builder.build(&stmt))
            .stream(&self.db)
            .await
            .map_err(IndexerError::DatabaseError)?;

        let mut result = IndexedOutputs {
            output_ids: vec![],
            milestone_index: 0.into(),
            page_size: 0,              // TODO
            next_offset: None,         // TODO
        };

        if let Some(item) = stream.try_next().await.map_err(IndexerError::DatabaseError)? {
            result.milestone_index = item.current_milestone_index.into();
            let bytes: [u8; OutputId::LENGTH] = item.output_id.try_into().unwrap();
            result.output_ids.push(bytes.try_into().unwrap());
        }

        while let Some(item) = stream.try_next().await.map_err(IndexerError::DatabaseError)? {
            // TODO: Reconsider safety implications
            let bytes: [u8; OutputId::LENGTH] = item.output_id.try_into().unwrap();
            result.output_ids.push(bytes.try_into().unwrap());
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
    pub page_size: u64,
    pub offset: Option<Vec<u8>>,
}

#[derive(Debug, FromQueryResult)]
pub struct JoinedResult {
    pub output_id: AddressDb,
    pub cursor: String,
    pub current_milestone_index: MilestoneIndexDb,
}

#[derive(Debug)]
pub struct IndexedOutputs {
    pub output_ids: Vec<OutputId>,
    pub milestone_index: MilestoneIndex,
    pub page_size: u64,
    pub next_offset: Option<Vec<u8>>,
}

#[cfg(test)]
mod test {
    use super::*;

    use chrono::NaiveDate;

    #[test]
    fn check_offset_length() {
        assert_eq!(OFFSET_LENGTH, 38);
    }

    #[test]
    fn offset_to_date_time() {
        let offset = &42u32.to_le_bytes();
        let expected = NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 42);
        let result = offset_to_naive_date_time(offset).unwrap();
        assert_eq!(result, expected);
    }

    // TODO: Testcase for max amount transaction
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod alias;
pub(crate) mod error;
pub(crate) mod extended;
pub(crate) mod foundry;
pub(crate) mod nft;

use chrono::Local;

use bee_ledger::workers::event::{OutputConsumed, OutputCreated};
use bee_message::{
    address::Address,
    milestone::MilestoneIndex,
    output::{Output, OutputId},
};

use sea_orm::{
    prelude::*, ActiveModelTrait, Condition, ConnectionTrait, Database, DatabaseConnection, EntityTrait, NotSet,
    QueryOrder, QuerySelect, Schema, Set,
};

use chrono::NaiveDateTime;

pub use alias::Model as AliasAdapter;
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
    pub async fn new() -> Self {
        let db = Database::connect("sqlite:memory:").await.unwrap();

        let builder = db.get_database_backend();
        let schema = Schema::new(builder);

        builder.build(&schema.create_table_from_entity(alias::Entity));

        // TODO: Create indices!

        Self { db }
    }

    pub async fn process_created_output(&self, created: &OutputCreated) -> Result<(), sea_orm::error::DbErr> {
        let output_id = created.output_id;
        let milestone_index = 42u32; // TODO: Add milestone index.

        match &created.output {
            Output::Alias(output) => {
                let alias = alias::ActiveModel {
                    // TODO: Use binary and let sqlite do the hex stuff
                    alias_id: Set(hex::encode(output.alias_id())),
                    output_id: Set(output_id.to_string()),
                    created_at: Set(Local::now().naive_local()),
                    amount: Set(output.amount() as i64), // TODO: Fix type?
                    state_controller: Set(hex::encode(output.state_controller())),
                    governor: Set(hex::encode(output.governor())),
                    issuer: NotSet,                               // TODO: Fix
                    sender: NotSet,                               // TODO: Fix
                    milestone_index: Set(milestone_index as i64), // TODO:
                };
                alias.insert(&self.db).await?;
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
    ) -> Result<Vec<AliasAdapter>, IndexerError> {
        let mut query = alias::Entity::find()
            .select_only()
            .column(alias::Column::OutputId)
            .column(alias::Column::CreatedAt)
            .order_by_asc(alias::Column::CreatedAt)
            .order_by_asc(alias::Column::OutputId);

        let mut condition = Condition::all();

        if options.page_size > 0 {
            if let Some(offset) = options.offset {
                if offset.len() != OFFSET_LENGTH {
                    return Err(IndexerError::InvalidOffsetLength(offset.len()));
                } else {
                    let (timestamp, output_id) = offset.split_at(MilestoneIndex::LENGTH);
                    let created_at = offset_to_naive_date_time(&timestamp)?;
                    condition = condition.add(alias::Column::CreatedAt.gte(created_at));
                    condition = condition.add(alias::Column::OutputId.gte(hex::encode(output_id)));
                }
            }
            query = query.limit(options.page_size + 1);
        }

        // TODO: Factor this out:

        if let Some(state_controller) = options.state_controller {
            let encoded = hex::encode(state_controller);
            condition = condition.add(alias::Column::StateController.eq(encoded));
        }
        if let Some(governor) = options.governor {
            let encoded = hex::encode(governor);
            condition = condition.add(alias::Column::Governor.eq(encoded));
        }
        if let Some(sender) = options.sender {
            let encoded = hex::encode(sender);
            condition = condition.add(alias::Column::Sender.eq(encoded));
        }
        if let Some(issuer) = options.issuer {
            let encoded = hex::encode(issuer);
            condition = condition.add(alias::Column::Issuer.eq(encoded));
        }

        // TODO: Pagination

        query
            .filter(condition)
            .all(&self.db)
            .await
            .map_err(IndexerError::DatabaseError)
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
}

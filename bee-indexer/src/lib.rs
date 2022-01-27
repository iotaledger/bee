// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod alias;
pub(crate) mod database;
pub(crate) mod extended;
pub(crate) mod foundry;
pub(crate) mod nft;

use alias::AliasAdapter;
use database::Database;

use bee_ledger::workers::event::{OutputConsumed, OutputCreated};
use bee_message::output::Output;

pub struct Indexer {
    db: Database,
}

impl Indexer {
    pub async fn process_created_output(&self, created: &OutputCreated) -> Result<(), sqlx::Error> {
        let output_id = created.output_id;
        let milestone_index = 42.into(); // TODO: Add milestone index.

        match &created.output {
            Output::Alias(alias_output) => {
                let alias = AliasAdapter::from_alias_output_with_id(alias_output, output_id, milestone_index);
                self.db.insert_alias(alias).await?;
            }
            _ => todo!(),
        }

        Ok(())
    }

    pub async fn process_spent_output(&self, consumed: &OutputConsumed) -> Result<(), sqlx::Error> {
        match &consumed.output {
            Output::Alias(alias_output) => {
                self.db.remove_alias(&hex::encode(alias_output.alias_id())).await?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}

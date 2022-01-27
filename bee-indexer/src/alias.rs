// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::database::Table;

use bee_message::{
    milestone::MilestoneIndex,
    output::{AliasOutput, OutputId},
};
#[derive(Clone, Debug, Eq, sqlx::FromRow, PartialEq)]
pub(crate) struct AliasAdapter {
    pub alias_id: String,
    pub output_id: String,
    pub amount: i64,
    pub state_controller: String,
    pub governor: String,
    pub issuer: Option<String>,
    pub sender: Option<String>,
    pub milestone_index: i64,
}

impl AliasAdapter {
    pub(crate) fn from_alias_output_with_id(
        alias: &AliasOutput,
        output_id: OutputId,
        milestone_index: MilestoneIndex,
    ) -> Self {
        Self {
            alias_id: hex::encode(alias.alias_id()),
            output_id: output_id.to_string(), // TODO: Fix once `OutputCreated` is finalized
            amount: alias.amount() as i64,    // TODO: Fix
            state_controller: hex::encode(alias.state_controller()),
            governor: hex::encode(alias.governor()),
            issuer: None, // TODO: Fix
            sender: None, // TODO: Fix,
            milestone_index: *milestone_index as i64,
        }
    }
}

impl Table for AliasAdapter {
    const TABLE_NAME: &'static str = "alias_outputs";
}

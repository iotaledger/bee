// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;
use bee_rest_api::types::dtos::OutputDto;

#[derive(Serialize)]
pub struct MilestonePayload {
    pub index: u32,
    pub timestamp: u64,
}

pub struct Message {
    pub bytes: Vec<u8>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub bytes: Vec<u8>,
}

#[derive(Serialize)]
pub struct NewIndexationMessageResponse {
    pub id: String,
    pub index: Vec<u8>,
}

#[derive(Serialize)]
pub struct CreatedOutputResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "outputIndex")]
    pub output_index: String,
    #[serde(rename = "isSpent")]
    pub is_spent: bool,
    pub output: OutputDto,
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        self.bytes
    }
}

#[derive(Serialize)]
pub struct MessageMetadata {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "parentMessageIds")]
    pub parent_message_ids: Vec<String>,
    #[serde(rename = "isSolid")]
    pub is_solid: bool,
    #[serde(rename = "referencedByMilestoneIndex")]
    pub referenced_by_milestone_index: u32,
    #[serde(rename = "ledgerInclusionState")]
    pub ledger_inclusion_state: LedgerInclusionState,
    #[serde(rename = "shouldPromote")]
    pub should_promote: bool,
    #[serde(rename = "shouldReattach")]
    pub should_reattach: bool,
}

pub enum LedgerInclusionState {
    NoTransaction,
    #[allow(dead_code)]
    Conflicting,
    #[allow(dead_code)]
    Included,
}

impl Serialize for LedgerInclusionState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use LedgerInclusionState::*;

        let s = match *self {
            NoTransaction => "noTransaction",
            Conflicting => "conflicting",
            Included => "included",
        };
        serializer.serialize_str(s)
    }
}

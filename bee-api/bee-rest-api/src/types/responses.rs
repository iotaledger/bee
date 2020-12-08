// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{MessageDto, OutputDto};
use serde::Serialize;

/// Marker trait for data bodies.
pub trait DataBody {}

/// Data response.
#[derive(Clone, Debug, Serialize)]
pub struct DataResponse<T: DataBody> {
    pub data: T,
}

impl<T: DataBody> DataResponse<T> {
    /// Create a new data response.
    pub(crate) fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

/// Error response.
#[derive(Clone, Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

impl ErrorResponse {
    /// Create a new error response.
    pub(crate) fn new(error: ErrorBody) -> Self {
        Self { error }
    }
}

/// Response of GET /api/v1/info
#[derive(Clone, Debug, Serialize)]
pub struct GetInfoResponse {
    pub name: String,
    pub version: String,
    #[serde(rename = "isHealthy")]
    pub is_healthy: bool,
    #[serde(rename = "networkId")]
    pub network_id: String,
    #[serde(rename = "latestMilestoneIndex")]
    pub latest_milestone_index: u32,
    #[serde(rename = "solidMilestoneIndex")]
    pub solid_milestone_index: u32,
    #[serde(rename = "pruningIndex")]
    pub pruning_index: u32,
    pub features: Vec<String>,
}

impl DataBody for GetInfoResponse {}

/// Response of GET /api/v1/tips
#[derive(Clone, Debug, Serialize)]
pub struct GetTipsResponse {
    #[serde(rename = "tip1MessageId")]
    pub tip_1_message_id: String,
    #[serde(rename = "tip2MessageId")]
    pub tip_2_message_id: String,
}

impl DataBody for GetTipsResponse {}

/// Response of POST /api/v1/messages
#[derive(Clone, Debug, Serialize)]
pub struct PostMessageResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

impl DataBody for PostMessageResponse {}

/// Response of GET /api/v1/messages/{message_id}?index={INDEX}
#[derive(Clone, Debug, Serialize)]
pub struct GetMessagesByIndexResponse {
    pub index: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
}

impl DataBody for GetMessagesByIndexResponse {}

/// Response of GET /api/v1/messages/{message_id}
#[derive(Clone, Debug, Serialize)]
pub struct GetMessageResponse(pub MessageDto);

impl DataBody for GetMessageResponse {}

/// Response of GET /api/v1/messages/{message_id}/metadata
#[derive(Clone, Debug, Serialize)]
pub struct GetMessageMetadataResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "parent1MessageId")]
    pub parent_1_message_id: String,
    #[serde(rename = "parent2MessageId")]
    pub parent_2_message_id: String,
    #[serde(rename = "isSolid")]
    pub is_solid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "referencedByMilestoneIndex")]
    pub referenced_by_milestone_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ledgerInclusionState")]
    pub ledger_inclusion_state: Option<LedgerInclusionStateDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldPromote")]
    pub should_promote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldReattach")]
    pub should_reattach: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum LedgerInclusionStateDto {
    Conflicting,
    Included,
    NoTransaction,
}

impl DataBody for GetMessageMetadataResponse {}

/// Response of GET /api/v1/messages/{message_id}/children
#[derive(Clone, Debug, Serialize)]
pub struct GetChildrenResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "childrenMessageIds")]
    pub children_message_ids: Vec<String>,
}

impl DataBody for GetChildrenResponse {}

/// Response of GET /api/v1/milestone/{milestone_index}
#[derive(Clone, Debug, Serialize)]
pub struct GetMilestoneResponse {
    #[serde(rename = "milestoneIndex")]
    pub milestone_index: u32,
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub timestamp: u64,
}

impl DataBody for GetMilestoneResponse {}

/// Response of GET /api/v1/outputs/{output_id}
#[derive(Clone, Debug, Serialize)]
pub struct GetOutputByOutputIdResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "outputIndex")]
    pub output_index: u16,
    #[serde(rename = "isSpent")]
    pub is_spent: bool,
    pub output: OutputDto,
}

impl DataBody for GetOutputByOutputIdResponse {}

/// Response of GET /api/v1/addresses/{address}
#[derive(Clone, Debug, Serialize)]
pub struct GetBalanceForAddressResponse {
    // The type of the address (0=WOTS, 1=Ed25519).
    #[serde(rename = "type")]
    pub kind: u8,
    // hex encoded address
    pub address: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    pub balance: u64,
}

impl DataBody for GetBalanceForAddressResponse {}

/// Response of GET /api/v1/addresses/{address}/outputs
#[derive(Clone, Debug, Serialize)]
pub struct GetOutputsForAddressResponse {
    pub address: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "outputIds")]
    pub output_ids: Vec<String>,
}

impl DataBody for GetOutputsForAddressResponse {}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
    #[allow(dead_code)]
    /// Get the body of the response.
    pub(crate) fn body(&self) -> &T {
        &self.data
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: &'static str,
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
    #[allow(dead_code)]
    /// Get the body of the response.
    pub(crate) fn body(&self) -> &ErrorBody {
        &self.error
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

#[derive(Clone, Debug, Serialize)]
pub struct MessageDto {
    #[serde(rename = "networkId")]
    pub network_id: String,
    #[serde(rename = "parent1MessageId")]
    pub parent_1_message_id: String,
    #[serde(rename = "parent2MessageId")]
    pub parent_2_message_id: String,
    pub payload: Option<PayloadDto>,
    pub nonce: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum PayloadDto {
    Transaction(TransactionPayloadDto),
    Indexation(IndexationPayloadDto),
    Milestone(MilestonePayloadDto),
}

#[derive(Clone, Debug, Serialize)]
pub struct TransactionPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub essence: TransactionEssenceDto,
    #[serde(rename = "unlockBlocks")]
    pub unlock_blocks: Vec<UnlockBlockDto>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TransactionEssenceDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub inputs: Vec<UtxoInputDto>,
    pub outputs: Vec<OutputDto>,
    pub payload: Option<IndexationPayloadDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum OutputDto {
    SignatureLockedSingle(SignatureLockedSingleOutputDto),
}

#[derive(Clone, Debug, Serialize)]
pub struct UtxoInputDto {
    #[serde(rename = "type")]
    pub kind: u32,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "transactionOutputIndex")]
    pub transaction_output_index: u16,
}

#[derive(Clone, Debug, Serialize)]
pub struct SignatureLockedSingleOutputDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub address: Ed25519AddressDto,
    pub amount: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Ed25519AddressDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub address: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum UnlockBlockDto {
    Signature(SignatureUnlockBlockDto),
    Reference(ReferenceUnlockBlockDto),
}

#[derive(Clone, Debug, Serialize)]
pub struct SignatureUnlockBlockDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub signature: Ed25519SignatureDto,
}

#[derive(Clone, Debug, Serialize)]
pub struct Ed25519SignatureDto {
    #[serde(rename = "type")]
    pub kind: u32,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReferenceUnlockBlockDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub reference: u16,
}

#[derive(Clone, Debug, Serialize)]
pub struct IndexationPayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: String,
    pub data: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct MilestonePayloadDto {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: u32,
    pub timestamp: u64,
    #[serde(rename = "inclusionMerkleProof")]
    pub inclusion_merkle_proof: String,
    pub signatures: Vec<String>,
}

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
    pub address: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    pub balance: u32,
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

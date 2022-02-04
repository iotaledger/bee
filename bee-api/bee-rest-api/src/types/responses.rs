// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{
    body::BodyInner,
    dtos::{LedgerInclusionStateDto, MessageDto, OutputDto, PeerDto, ReceiptDto},
};

use serde::{Deserialize, Serialize};

/// Response of GET /api/v1/info.
/// Returns general information about the node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    #[serde(rename = "isHealthy")]
    pub is_healthy: bool,
    #[serde(rename = "networkId")]
    pub network_id: String,
    #[serde(rename = "bech32HRP")]
    pub bech32_hrp: String,
    #[serde(rename = "minPoWScore")]
    pub min_pow_score: f64,
    #[serde(rename = "messagesPerSecond")]
    pub messages_per_second: f64,
    #[serde(rename = "referencedMessagesPerSecond")]
    pub referenced_messages_per_second: f64,
    #[serde(rename = "referencedRate")]
    pub referenced_rate: f64,
    #[serde(rename = "latestMilestoneTimestamp")]
    pub latest_milestone_timestamp: u64,
    #[serde(rename = "latestMilestoneIndex")]
    pub latest_milestone_index: u32,
    #[serde(rename = "confirmedMilestoneIndex")]
    pub confirmed_milestone_index: u32,
    #[serde(rename = "pruningIndex")]
    pub pruning_index: u32,
    pub features: Vec<String>,
}

impl BodyInner for InfoResponse {}

/// Response of GET /api/v1/tips.
/// Returns non-lazy tips.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TipsResponse {
    #[serde(rename = "tipMessageIds")]
    pub tip_message_ids: Vec<String>,
}

impl BodyInner for TipsResponse {}

/// Response of POST /api/v1/messages.
/// Returns the message identifier of the submitted message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitMessageResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

impl BodyInner for SubmitMessageResponse {}

/// Response of GET /api/v1/messages?index={INDEX}.
/// Returns all messages ids that match a given indexation key.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessagesFindResponse {
    pub index: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
}

impl BodyInner for MessagesFindResponse {}

/// Response of GET /api/v1/messages/{message_id}.
/// Returns a specific message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageResponse(pub MessageDto);

impl BodyInner for MessageResponse {}

/// Response of GET /api/v1/messages/{message_id}/metadata.
/// Returns the metadata of a message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageMetadataResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "parentMessageIds")]
    pub parent_message_ids: Vec<String>,
    #[serde(rename = "isSolid")]
    pub is_solid: bool,
    #[serde(rename = "referencedByMilestoneIndex", skip_serializing_if = "Option::is_none")]
    pub referenced_by_milestone_index: Option<u32>,
    #[serde(rename = "milestoneIndex", skip_serializing_if = "Option::is_none")]
    pub milestone_index: Option<u32>,
    #[serde(rename = "ledgerInclusionState", skip_serializing_if = "Option::is_none")]
    pub ledger_inclusion_state: Option<LedgerInclusionStateDto>,
    #[serde(rename = "conflictReason", skip_serializing_if = "Option::is_none")]
    pub conflict_reason: Option<u8>,
    #[serde(rename = "shouldPromote", skip_serializing_if = "Option::is_none")]
    pub should_promote: Option<bool>,
    #[serde(rename = "shouldReattach", skip_serializing_if = "Option::is_none")]
    pub should_reattach: Option<bool>,
}

impl BodyInner for MessageMetadataResponse {}

/// Response of GET /api/v1/messages/{message_id}/children.
/// Returns all children of a specific message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageChildrenResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "childrenMessageIds")]
    pub children_message_ids: Vec<String>,
}

impl BodyInner for MessageChildrenResponse {}

/// Response of GET /api/v1/outputs/{output_id}.
/// Returns all information about a specific output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "outputIndex")]
    pub output_index: u16,
    #[serde(rename = "isSpent")]
    pub is_spent: bool,
    pub output: OutputDto,
    #[serde(rename = "ledgerIndex", default)]
    pub ledger_index: u32,
    #[serde(rename = "milestoneIndexSpent", skip_serializing_if = "Option::is_none")]
    pub milestone_index_spent: Option<u32>,
    #[serde(rename = "transactionIdSpent", skip_serializing_if = "Option::is_none")]
    pub transaction_id_spent: Option<String>,
}

impl BodyInner for OutputResponse {}

/// Response of GET /api/v1/addresses/{address}.
/// Returns information about an address.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BalanceAddressResponse {
    #[serde(rename = "addressType")]
    pub address_type: u8,
    pub address: String,
    pub balance: u64,
    #[serde(rename = "dustAllowed")]
    pub dust_allowed: bool,
    #[serde(rename = "ledgerIndex", default)]
    pub ledger_index: u32,
}

impl BodyInner for BalanceAddressResponse {}

/// Response of GET /api/v1/addresses/{address}/outputs.
/// Returns the outputs of an address.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputsAddressResponse {
    #[serde(rename = "addressType")]
    pub address_type: u8,
    pub address: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "outputIds")]
    pub output_ids: Vec<String>,
    #[serde(rename = "ledgerIndex", default)]
    pub ledger_index: u32,
}

impl BodyInner for OutputsAddressResponse {}

/// Response of:
/// * GET /api/v1/receipts/{milestone_index}, returns all stored receipts for the given milestone index.
/// * GET /api/v1/receipts, returns all stored receipts, independent of a milestone index.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptsResponse {
    pub receipts: Vec<ReceiptDto>,
}

impl BodyInner for ReceiptsResponse {}

/// Response of GET /api/v1/treasury.
/// Returns all information about the treasury.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryResponse {
    #[serde(rename = "milestoneId")]
    pub milestone_id: String,
    pub amount: u64,
}

impl BodyInner for TreasuryResponse {}

/// Response of GET /api/v1/milestone/{milestone_index}.
/// Returns information about a milestone.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneResponse {
    #[serde(rename = "index")]
    pub milestone_index: u32,
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub timestamp: u64,
}

impl BodyInner for MilestoneResponse {}

/// Response of GET /api/v1/milestone/{milestone_index}/utxo-changes.
/// Returns all UTXO changes that happened at a specific milestone.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtxoChangesResponse {
    pub index: u32,
    #[serde(rename = "createdOutputs")]
    pub created_outputs: Vec<String>,
    #[serde(rename = "consumedOutputs")]
    pub consumed_outputs: Vec<String>,
}

impl BodyInner for UtxoChangesResponse {}

/// Response of GET /api/v1/peers.
/// Returns information about all peers of the node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeersResponse(pub Vec<PeerDto>);

impl BodyInner for PeersResponse {}

/// Response of POST /api/v1/peers.
/// Returns information about the added peer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddPeerResponse(pub PeerDto);

impl BodyInner for AddPeerResponse {}

/// Response of GET /api/v1/peer/{peer_id}.
/// Returns information about a specific peer of the node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerResponse(pub PeerDto);

impl BodyInner for PeerResponse {}

/// Response of GET /api/plugins/debug/whiteflag.
/// Returns the computed merkle tree hash for the given white flag traversal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WhiteFlagResponse {
    #[serde(rename = "merkleTreeHash")]
    pub merkle_tree_hash: String,
}

impl BodyInner for WhiteFlagResponse {}

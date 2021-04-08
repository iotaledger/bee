// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{
    body::BodyInner,
    dtos::{LedgerInclusionStateDto, MessageDto, OutputDto, PeerDto, ReceiptDto},
};

use serde::{Deserialize, Serialize};

/// Response of GET /debug/whiteflag
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WhiteFlagResponse {
    #[serde(rename = "merkleTreeHash")]
    pub merkle_tree_hash: String,
}

impl BodyInner for WhiteFlagResponse {}

/// Response of GET /api/v1/treasury
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreasuryResponse {
    #[serde(rename = "milestoneId")]
    pub milestone_id: String,
    pub amount: u64,
}

impl BodyInner for TreasuryResponse {}

/// Response of GET /api/v1/tips
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TipsResponse {
    #[serde(rename = "tipMessageIds")]
    pub tip_message_ids: Vec<String>,
}

impl BodyInner for TipsResponse {}

/// Response of POST /api/v1/messages
#[derive(Clone, Debug, Serialize)]
pub struct SubmitMessageResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

impl BodyInner for SubmitMessageResponse {}

/// Response of GET /api/v1/receipts/{milestone_index} and /api/v1/receipts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptsResponse(pub Vec<ReceiptDto>);

impl BodyInner for ReceiptsResponse {}

/// Response of POST /api/v1/peers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddPeerResponse(pub PeerDto);

impl BodyInner for AddPeerResponse {}

/// Response of GET /api/v1/addresses/{address}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BalanceAddressResponse {
    // The type of the address (1=Ed25519).
    #[serde(rename = "addressType")]
    pub address_type: u8,
    // hex encoded address
    pub address: String,
    pub balance: u64,
    #[serde(rename = "dustAllowed")]
    pub dust_allowed: bool,
}

impl BodyInner for BalanceAddressResponse {}

/// Response of GET /api/v1/info
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
    #[serde(rename = "minPowScore")]
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

/// Response of GET /api/v1/messages/{message_id}/children
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

/// Response of GET /api/v1/messages/{message_id}/metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageMetadataResponse {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "parentMessageIds")]
    pub parent_message_ids: Vec<String>,
    #[serde(rename = "isSolid")]
    pub is_solid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "referencedByMilestoneIndex")]
    pub referenced_by_milestone_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "milestoneIndex")]
    pub milestone_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ledgerInclusionState")]
    pub ledger_inclusion_state: Option<LedgerInclusionStateDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "conflictReason")]
    pub conflict_reason: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldPromote")]
    pub should_promote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldReattach")]
    pub should_reattach: Option<bool>,
}

impl BodyInner for MessageMetadataResponse {}

/// Response of GET /api/v1/messages/{message_id}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageResponse(pub MessageDto);

impl BodyInner for MessageResponse {}

/// Response of GET /api/v1/messages/{message_id}?index={INDEX}
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

/// Response of GET /api/v1/milestone/{milestone_index}/utxo-changes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtxoChangesResponse {
    pub index: u32,
    #[serde(rename = "createdOutputs")]
    pub created_outputs: Vec<String>,
    #[serde(rename = "consumedOutputs")]
    pub consumed_outputs: Vec<String>,
}

impl BodyInner for UtxoChangesResponse {}

/// Response of GET /api/v1/milestone/{milestone_index}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneResponse {
    #[serde(rename = "index")]
    pub milestone_index: u32,
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub timestamp: u64,
}

impl BodyInner for MilestoneResponse {}

/// Response of GET /api/v1/outputs/{output_id}
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
}

impl BodyInner for OutputResponse {}

/// Response of GET /api/v1/addresses/{address}/outputs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputsAddressResponse {
    // The type of the address (1=Ed25519).
    #[serde(rename = "addressType")]
    pub address_type: u8,
    pub address: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    #[serde(rename = "outputIds")]
    pub output_ids: Vec<String>,
}

impl BodyInner for OutputsAddressResponse {}

/// Response of GET /api/v1/peer/{peer_id}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerResponse(pub PeerDto);

impl BodyInner for PeerResponse {}

/// Response of GET /api/v1/info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeersResponse(pub Vec<PeerDto>);

impl BodyInner for PeersResponse {}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{output::dto::OutputDto, payload::dto::MilestonePayloadDto, BlockDto};
use serde::{Deserialize, Serialize};

use crate::types::{
    body::BodyInner,
    dtos::{LedgerInclusionStateDto, PeerDto, ReceiptDto},
};

/// Response of GET /api/v2/info.
/// Returns general information about the node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    pub status: StatusResponse,
    pub protocol: ProtocolResponse,
    #[serde(rename = "baseToken")]
    pub base_token: BaseTokenResponse,
    pub metrics: MetricsResponse,
    pub features: Vec<String>,
    pub plugins: Vec<String>,
}

impl BodyInner for InfoResponse {}

/// Returned in [`InfoResponse`].
/// Status information about the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatusResponse {
    #[serde(rename = "isHealthy")]
    pub is_healthy: bool,
    #[serde(rename = "latestMilestone")]
    pub latest_milestone: LatestMilestoneResponse,
    #[serde(rename = "confirmedMilestone")]
    pub confirmed_milestone: ConfirmedMilestoneResponse,
    #[serde(rename = "pruningIndex")]
    pub pruning_index: u32,
}

/// Returned in [`StatusResponse`].
/// Information about the latest milestone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LatestMilestoneResponse {
    #[serde(rename = "index")]
    pub index: u32,
    #[serde(rename = "timestamp")]
    pub timestamp: u32,
    #[serde(rename = "milestoneId")]
    pub milestone_id: String,
}

/// Returned in [`StatusResponse`].
/// Information about the confirmed milestone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfirmedMilestoneResponse {
    #[serde(rename = "index")]
    pub index: u32,
    #[serde(rename = "timestamp")]
    pub timestamp: u32,
    #[serde(rename = "milestoneId")]
    pub milestone_id: String,
}

/// Returned in [`InfoResponse`].
/// Protocol information about the node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProtocolResponse {
    pub version: u8,
    #[serde(rename = "networkName")]
    pub network_name: String,
    #[serde(rename = "bech32HRP")]
    pub bech32_hrp: String,
    #[serde(rename = "minPoWScore")]
    pub min_pow_score: f64,
    #[serde(rename = "rentStructure")]
    pub rent_structure: RentStructureResponse,
    #[serde(rename = "tokenSupply")]
    pub token_supply: String,
}

/// Returned in [`InfoResponse`].
/// Information about the base token.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BaseTokenResponse {
    pub name: String,
    #[serde(rename = "tickerSymbol")]
    pub ticker_symbol: String,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subunit: Option<String>,
    pub decimals: u8,
    #[serde(rename = "useMetricPrefix")]
    pub use_metric_prefix: bool,
}

/// Returned in [`InfoResponse`].
/// Rent information about the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RentStructureResponse {
    #[serde(rename = "vByteCost")]
    pub v_byte_cost: u64,
    #[serde(rename = "vByteFactorKey")]
    pub v_byte_factor_key: u64,
    #[serde(rename = "vByteFactorData")]
    pub v_byte_factor_data: u64,
}

/// Returned in [`InfoResponse`].
/// Metric information about the node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricsResponse {
    #[serde(rename = "blocksPerSecond")]
    pub blocks_per_second: f64,
    #[serde(rename = "referencedBlocksPerSecond")]
    pub referenced_blocks_per_second: f64,
    #[serde(rename = "referencedRate")]
    pub referenced_rate: f64,
}

/// Response of GET /api/v2/tips.
/// Returns non-lazy tips.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TipsResponse {
    pub tips: Vec<String>,
}

impl BodyInner for TipsResponse {}

/// Response of POST /api/v2/blocks.
/// Returns the block identifier of the submitted block.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubmitBlockResponse {
    #[serde(rename = "blockId")]
    pub block_id: String,
}

impl BodyInner for SubmitBlockResponse {}

/// Response of GET /api/v2/blocks/{block_id}.
/// Returns a specific block.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockResponse(pub BlockDto);

impl BodyInner for BlockResponse {}

/// Response of GET /api/v2/blocks/{block_id}/metadata.
/// Returns the metadata of a block.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockMetadataResponse {
    #[serde(rename = "blockId")]
    pub block_id: String,
    pub parents: Vec<String>,
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

impl BodyInner for BlockMetadataResponse {}

/// Response of GET /api/v2/blocks/{block_id}/children.
/// Returns all children of a specific block.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockChildrenResponse {
    #[serde(rename = "blockId")]
    pub block_id: String,
    #[serde(rename = "maxResults")]
    pub max_results: usize,
    pub count: usize,
    pub children: Vec<String>,
}

impl BodyInner for BlockChildrenResponse {}

/// Response of GET /api/v2/outputs/{output_id}.
/// Returns an output and its metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputResponse {
    pub metadata: OutputMetadataResponse,
    pub output: OutputDto,
}

/// Response of GET /api/v2/outputs/{output_id}/metadata.
/// Returns an output metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputMetadataResponse {
    #[serde(rename = "blockId")]
    pub block_id: String,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "outputIndex")]
    pub output_index: u16,
    #[serde(rename = "isSpent")]
    pub is_spent: bool,
    #[serde(rename = "milestoneIndexSpent", skip_serializing_if = "Option::is_none")]
    pub milestone_index_spent: Option<u32>,
    #[serde(rename = "milestoneTimestampSpent", skip_serializing_if = "Option::is_none")]
    pub milestone_timestamp_spent: Option<u32>,
    #[serde(rename = "transactionIdSpent", skip_serializing_if = "Option::is_none")]
    pub transaction_id_spent: Option<String>,
    #[serde(rename = "milestoneIndexBooked")]
    pub milestone_index_booked: u32,
    #[serde(rename = "milestoneTimestampBooked")]
    pub milestone_timestamp_booked: u32,
    #[serde(rename = "ledgerIndex", default)]
    pub ledger_index: u32,
}

impl BodyInner for OutputResponse {}

/// Response of:
/// * GET /api/v2/receipts/{milestone_index}, returns all stored receipts for the given milestone index.
/// * GET /api/v2/receipts, returns all stored receipts, independent of a milestone index.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReceiptsResponse {
    pub receipts: Vec<ReceiptDto>,
}

impl BodyInner for ReceiptsResponse {}

/// Response of GET /api/v2/treasury.
/// Returns all information about the treasury.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TreasuryResponse {
    #[serde(rename = "milestoneId")]
    pub milestone_id: String,
    pub amount: String,
}

impl BodyInner for TreasuryResponse {}

/// Response of GET /api/v2/milestone/{milestone_index}.
/// Returns information about a milestone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MilestoneResponse(pub MilestonePayloadDto);

impl BodyInner for MilestoneResponse {}

/// Response of GET /api/v2/milestone/{milestone_index}/utxo-changes.
/// Returns all UTXO changes that happened at a specific milestone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UtxoChangesResponse {
    pub index: u32,
    #[serde(rename = "createdOutputs")]
    pub created_outputs: Vec<String>,
    #[serde(rename = "consumedOutputs")]
    pub consumed_outputs: Vec<String>,
}

impl BodyInner for UtxoChangesResponse {}

/// Response of GET /api/v2/peers.
/// Returns information about all peers of the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeersResponse(pub Vec<PeerDto>);

impl BodyInner for PeersResponse {}

/// Response of POST /api/v2/peers.
/// Returns information about the added peer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AddPeerResponse(pub PeerDto);

impl BodyInner for AddPeerResponse {}

/// Response of GET /api/v2/peer/{peer_id}.
/// Returns information about a specific peer of the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerResponse(pub PeerDto);

impl BodyInner for PeerResponse {}

/// Response of GET /api/plugins/debug/whiteflag.
/// Returns the computed merkle tree hash for the given white flag traversal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhiteFlagResponse {
    #[serde(rename = "merkleTreeHash")]
    pub merkle_tree_hash: String,
}

impl BodyInner for WhiteFlagResponse {}

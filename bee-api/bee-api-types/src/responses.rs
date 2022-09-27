// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    output::{dto::OutputDto, RentStructure, RentStructureBuilder},
    payload::dto::MilestonePayloadDto,
    protocol::ProtocolParameters,
    BlockDto,
};
use serde::{Deserialize, Serialize};

use crate::{
    body::BodyInner,
    dtos::{LedgerInclusionStateDto, PeerDto, ReceiptDto},
    error::Error,
};

/// Response of GET /api/core/v2/info.
/// Returns general information about the node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    pub status: StatusResponse,
    #[serde(rename = "supportedProtocolVersions")]
    pub supported_protocol_versions: Vec<u8>,
    pub protocol: ProtocolResponse,
    #[serde(rename = "pendingProtocolParameters")]
    pub pending_protocol_parameters: Vec<PendingProtocolParameter>,
    #[serde(rename = "baseToken")]
    pub base_token: BaseTokenResponse,
    pub metrics: MetricsResponse,
    pub features: Vec<String>,
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
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u32>,
    #[serde(rename = "milestoneId", skip_serializing_if = "Option::is_none")]
    pub milestone_id: Option<String>,
}

/// Returned in [`StatusResponse`].
/// Information about the confirmed milestone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfirmedMilestoneResponse {
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u32>,
    #[serde(rename = "milestoneId", skip_serializing_if = "Option::is_none")]
    pub milestone_id: Option<String>,
}

/// Returned in [`InfoResponse`].
/// Protocol information about the node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolResponse {
    pub version: u8,
    #[serde(rename = "networkName")]
    pub network_name: String,
    #[serde(rename = "bech32Hrp")]
    pub bech32_hrp: String,
    #[serde(rename = "minPowScore")]
    pub min_pow_score: u32,
    #[serde(rename = "belowMaxDepth")]
    pub below_max_depth: u8,
    #[serde(rename = "rentStructure")]
    pub rent_structure: RentStructureResponse,
    #[serde(rename = "tokenSupply")]
    pub token_supply: String,
}

impl TryFrom<ProtocolResponse> for ProtocolParameters {
    type Error = Error;

    fn try_from(response: ProtocolResponse) -> Result<Self, Self::Error> {
        Ok(ProtocolParameters::new(
            response.version,
            response.network_name,
            response.bech32_hrp,
            response.min_pow_score,
            response.below_max_depth,
            response.rent_structure.into(),
            response
                .token_supply
                .parse()
                .map_err(|_| Error::InvalidField("token_supply"))?,
        )?)
    }
}

/// Returned in [`InfoResponse`].
/// Pending protocol parameters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingProtocolParameter {
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(rename = "targetMilestoneIndex")]
    pub target_milestone_index: u32,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: u8,
    pub params: String,
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
    pub v_byte_cost: u32,
    #[serde(rename = "vByteFactorKey")]
    pub v_byte_factor_key: u8,
    #[serde(rename = "vByteFactorData")]
    pub v_byte_factor_data: u8,
}

impl From<RentStructureResponse> for RentStructure {
    fn from(response: RentStructureResponse) -> Self {
        RentStructureBuilder::new()
            .byte_cost(response.v_byte_cost)
            .key_factor(response.v_byte_factor_key)
            .data_factor(response.v_byte_factor_data)
            .finish()
    }
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

/// Response of GET /api/core/v2/tips.
/// Returns non-lazy tips.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TipsResponse {
    pub tips: Vec<String>,
}

/// Response of POST /api/core/v2/blocks.
/// Returns the block identifier of the submitted block.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubmitBlockResponse {
    #[serde(rename = "blockId")]
    pub block_id: String,
}

/// Response of GET /api/core/v2/blocks/{block_id}.
/// Returns a specific block.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockResponse {
    Json(BlockDto),
    Raw(Vec<u8>),
}

/// Response of GET /api/core/v2/blocks/{block_id}/metadata.
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
    #[serde(rename = "whiteFlagIndex", skip_serializing_if = "Option::is_none")]
    pub white_flag_index: Option<u32>,
    #[serde(rename = "shouldPromote", skip_serializing_if = "Option::is_none")]
    pub should_promote: Option<bool>,
    #[serde(rename = "shouldReattach", skip_serializing_if = "Option::is_none")]
    pub should_reattach: Option<bool>,
}

/// Response of GET /api/core/v2/outputs/{output_id}.
/// Returns an output and its metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputResponse {
    pub metadata: OutputMetadataResponse,
    pub output: OutputDto,
}

/// Response of GET /api/core/v2/outputs/{output_id}/metadata.
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

/// Response of:
/// * GET /api/core/v2/receipts/{milestone_index}, returns all stored receipts for the given milestone index.
/// * GET /api/core/v2/receipts, returns all stored receipts, independent of a milestone index.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReceiptsResponse {
    pub receipts: Vec<ReceiptDto>,
}

/// Response of GET /api/core/v2/treasury.
/// Returns all information about the treasury.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TreasuryResponse {
    #[serde(rename = "milestoneId")]
    pub milestone_id: String,
    pub amount: String,
}

/// Response of GET /api/core/v2/milestone/{milestone_index}.
/// Returns information about a milestone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MilestoneResponse {
    Json(MilestonePayloadDto),
    Raw(Vec<u8>),
}

/// Response of GET /api/core/v2/milestone/{milestone_index}/utxo-changes.
/// Returns all UTXO changes that happened at a specific milestone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UtxoChangesResponse {
    pub index: u32,
    #[serde(rename = "createdOutputs")]
    pub created_outputs: Vec<String>,
    #[serde(rename = "consumedOutputs")]
    pub consumed_outputs: Vec<String>,
}

/// Response of GET /api/core/v2/peers.
/// Returns information about all peers of the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeersResponse(pub Vec<PeerDto>);

/// Response of POST /api/core/v2/peers.
/// Returns information about the added peer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AddPeerResponse(pub PeerDto);

/// Response of GET /api/core/v2/peer/{peer_id}.
/// Returns information about a specific peer of the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerResponse(pub PeerDto);

/// Response of GET /api/plugins/debug/whiteflag.
/// Returns the computed merkle tree hash for the given white flag traversal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WhiteFlagResponse {
    #[serde(rename = "merkleTreeHash")]
    pub merkle_tree_hash: String,
}

/// Response of GET /api/routes.
/// Returns the available API route groups of the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoutesResponse {
    pub routes: Vec<String>,
}

#[cfg(feature = "axum")]
mod axum_response {
    use axum::{
        body::BoxBody,
        http::StatusCode,
        response::{IntoResponse, Response},
        Json,
    };

    use super::*;

    /// Macro to implement `IntoResponse` for simple cases which can just be wrapped in JSON.
    macro_rules! impl_into_response {
        ($($t:ty),*) => ($(
            impl IntoResponse for $t {
                fn into_response(self) -> Response<BoxBody> {
                    Json(self).into_response()
                }
            }
        )*)
    }

    impl_into_response!(
        InfoResponse,
        TipsResponse,
        BlockMetadataResponse,
        OutputResponse,
        OutputMetadataResponse,
        ReceiptsResponse,
        TreasuryResponse,
        UtxoChangesResponse,
        AddPeerResponse,
        PeersResponse,
        PeerResponse,
        WhiteFlagResponse
    );

    impl IntoResponse for SubmitBlockResponse {
        fn into_response(self) -> Response<BoxBody> {
            (StatusCode::CREATED, Json(self)).into_response()
        }
    }

    impl IntoResponse for BlockResponse {
        fn into_response(self) -> Response<BoxBody> {
            match self {
                BlockResponse::Json(dto) => Json(dto).into_response(),
                BlockResponse::Raw(bytes) => bytes.into_response(),
            }
        }
    }

    impl IntoResponse for MilestoneResponse {
        fn into_response(self) -> Response<BoxBody> {
            match self {
                MilestoneResponse::Json(dto) => Json(dto).into_response(),
                MilestoneResponse::Raw(bytes) => bytes.into_response(),
            }
        }
    }
}

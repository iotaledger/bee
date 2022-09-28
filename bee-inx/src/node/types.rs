// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::{milestone::types::Milestone, raw::Raw, return_err_if_none};

/// The [`NodeStatus`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeStatus {
    /// Signals if the node is healthy.
    pub is_healthy: bool,
    /// Signals if the node is synced.
    pub is_synced: bool,
    /// Signals if the node is almost synced (within a configured range).
    pub is_almost_synced: bool,
    /// The latest milestone seen by the node.
    pub latest_milestone: Milestone,
    /// The last confirmed milestone.
    pub confirmed_milestone: Milestone,
    /// The current protocol parameters.
    pub current_protocol_parameters: ProtocolParameters,
    /// The tangle pruning index of the node.
    pub tangle_pruning_index: bee::payload::milestone::MilestoneIndex,
    /// The milestones pruning index of the node.
    pub milestones_pruning_index: bee::payload::milestone::MilestoneIndex,
    /// The ledger pruning index of the node.
    pub ledger_pruning_index: bee::payload::milestone::MilestoneIndex,
    /// The ledger index of the node.
    pub ledger_index: bee::payload::milestone::MilestoneIndex,
}

impl TryFrom<proto::NodeStatus> for NodeStatus {
    type Error = bee::InxError;

    fn try_from(value: proto::NodeStatus) -> Result<Self, Self::Error> {
        Ok(NodeStatus {
            is_healthy: value.is_healthy,
            is_synced: value.is_synced,
            is_almost_synced: value.is_almost_synced,
            latest_milestone: return_err_if_none!(value.latest_milestone).try_into()?,
            confirmed_milestone: return_err_if_none!(value.confirmed_milestone).try_into()?,
            current_protocol_parameters: return_err_if_none!(value.current_protocol_parameters).into(),
            tangle_pruning_index: value.tangle_pruning_index.into(),
            milestones_pruning_index: value.milestones_pruning_index.into(),
            ledger_pruning_index: value.ledger_pruning_index.into(),
            ledger_index: value.ledger_index.into(),
        })
    }
}

impl From<NodeStatus> for proto::NodeStatus {
    fn from(value: NodeStatus) -> Self {
        Self {
            is_healthy: value.is_healthy,
            is_synced: value.is_synced,
            is_almost_synced: value.is_almost_synced,
            latest_milestone: Some(value.latest_milestone.into()),
            confirmed_milestone: Some(value.confirmed_milestone.into()),
            current_protocol_parameters: Some(value.current_protocol_parameters.into()),
            tangle_pruning_index: value.tangle_pruning_index.0,
            milestones_pruning_index: value.milestones_pruning_index.0,
            ledger_pruning_index: value.ledger_pruning_index.0,
            ledger_index: value.ledger_index.0,
        }
    }
}

/// The [`NodeConfiguration`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeConfiguration {
    pub milestone_public_key_count: u32,
    pub milestone_key_ranges: Box<[MilestoneKeyRange]>,
    pub base_token: BaseToken,
    pub supported_protocol_versions: Box<[u8]>,
}

impl TryFrom<proto::NodeConfiguration> for NodeConfiguration {
    type Error = bee::InxError;

    fn try_from(value: proto::NodeConfiguration) -> Result<Self, Self::Error> {
        Ok(NodeConfiguration {
            milestone_public_key_count: value.milestone_public_key_count,
            milestone_key_ranges: value.milestone_key_ranges.into_iter().map(Into::into).collect(),
            base_token: return_err_if_none!(value.base_token).into(),
            supported_protocol_versions: value.supported_protocol_versions.into_iter().map(|v| v as u8).collect(),
        })
    }
}

impl From<NodeConfiguration> for proto::NodeConfiguration {
    fn from(value: NodeConfiguration) -> Self {
        Self {
            milestone_public_key_count: value.milestone_public_key_count,
            milestone_key_ranges: value
                .milestone_key_ranges
                .into_vec()
                .into_iter()
                .map(Into::into)
                .collect(),
            base_token: Some(value.base_token.into()),
            supported_protocol_versions: value
                .supported_protocol_versions
                .into_vec()
                .into_iter()
                .map(|v| v as _)
                .collect(),
        }
    }
}

/// The [`BaseToken`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseToken {
    pub name: String,
    pub ticker_symbol: String,
    pub unit: String,
    pub subunit: String,
    pub decimals: u32,
    pub use_metric_prefix: bool,
}

impl From<proto::BaseToken> for BaseToken {
    fn from(value: proto::BaseToken) -> Self {
        Self {
            name: value.name,
            ticker_symbol: value.ticker_symbol,
            unit: value.unit,
            subunit: value.subunit,
            decimals: value.decimals,
            use_metric_prefix: value.use_metric_prefix,
        }
    }
}

impl From<BaseToken> for proto::BaseToken {
    fn from(value: BaseToken) -> Self {
        Self {
            name: value.name,
            ticker_symbol: value.ticker_symbol,
            unit: value.unit,
            subunit: value.subunit,
            decimals: value.decimals,
            use_metric_prefix: value.use_metric_prefix,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneKeyRange {
    pub public_key: Box<[u8]>,
    pub start_index: bee::payload::milestone::MilestoneIndex,
    pub end_index: bee::payload::milestone::MilestoneIndex,
}

impl From<proto::MilestoneKeyRange> for MilestoneKeyRange {
    fn from(value: proto::MilestoneKeyRange) -> Self {
        Self {
            public_key: value.public_key.into_boxed_slice(),
            start_index: value.start_index.into(),
            end_index: value.end_index.into(),
        }
    }
}

impl From<MilestoneKeyRange> for proto::MilestoneKeyRange {
    fn from(value: MilestoneKeyRange) -> Self {
        Self {
            public_key: value.public_key.into_vec(),
            start_index: value.start_index.0,
            end_index: value.end_index.0,
        }
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolParameters {
    pub protocol_version: u8,
    pub params: Raw<bee::protocol::ProtocolParameters>,
}

impl From<proto::RawProtocolParameters> for ProtocolParameters {
    fn from(value: proto::RawProtocolParameters) -> Self {
        Self {
            protocol_version: value.protocol_version as u8,
            params: value.params.into(),
        }
    }
}

impl From<ProtocolParameters> for proto::RawProtocolParameters {
    fn from(value: ProtocolParameters) -> Self {
        Self {
            protocol_version: value.protocol_version as u32,
            params: value.params.data(),
        }
    }
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::{maybe_missing, Milestone, ProtocolParameters};

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
            latest_milestone: maybe_missing!(value.latest_milestone).try_into()?,
            confirmed_milestone: maybe_missing!(value.confirmed_milestone).try_into()?,
            current_protocol_parameters: maybe_missing!(value.current_protocol_parameters).into(),
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

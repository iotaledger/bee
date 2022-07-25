// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::{field, Milestone};

/// The [`NodeStatus`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeStatus {
    /// Signals if the node is healthy.
    pub is_healthy: bool,
    /// The latest milestone seen by the node.
    pub latest_milestone: Milestone,
    /// The last confirmed milestone.
    pub confirmed_milestone: Milestone,
    /// The tangle pruning index of the node.
    pub tangle_pruning_index: u32,
    /// The milestones pruning index of the node.
    pub milestones_pruning_index: u32,
    /// The ledger pruning index of the node.
    pub ledger_pruning_index: u32,
    /// The ledger index of the node.
    pub ledger_index: u32,
}

impl TryFrom<proto::NodeStatus> for NodeStatus {
    type Error = bee::InxError;

    fn try_from(value: proto::NodeStatus) -> Result<Self, Self::Error> {
        Ok(NodeStatus {
            is_healthy: value.is_healthy,
            latest_milestone: field!(value.latest_milestone).try_into()?,
            confirmed_milestone: field!(value.confirmed_milestone).try_into()?,
            tangle_pruning_index: value.tangle_pruning_index,
            milestones_pruning_index: value.milestones_pruning_index,
            ledger_pruning_index: value.ledger_pruning_index,
            ledger_index: value.ledger_index,
        })
    }
}

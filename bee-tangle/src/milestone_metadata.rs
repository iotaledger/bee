// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{payload::milestone::MilestoneId, BlockId};

/// Defines milestone metadata.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
pub struct MilestoneMetadata {
    block_id: BlockId,
    milestone_id: MilestoneId,
    timestamp: u32,
}

impl MilestoneMetadata {
    /// Creates a new [`MilestoneMetadata`].
    pub fn new(block_id: BlockId, milestone_id: MilestoneId, timestamp: u32) -> Self {
        Self {
            block_id,
            milestone_id,
            timestamp,
        }
    }

    /// Returns the block id of a [`MilestoneMetadata`].
    pub fn block_id(&self) -> &BlockId {
        &self.block_id
    }

    /// Returns the milestone id of a [`MilestoneMetadata`].
    pub fn milestone_id(&self) -> &MilestoneId {
        &self.milestone_id
    }

    /// Returns the timestamp of a [`MilestoneMetadata`].
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{payload::milestone::MilestoneIndex, BlockId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("pruning target index {selected} below minimum {minimum}")]
    InvalidTargetIndex {
        selected: MilestoneIndex,
        minimum: MilestoneIndex,
    },
    #[error("missing snapshot info")]
    MissingSnapshotInfo,
    #[error("missing milestone {0}")]
    MissingMilestone(MilestoneIndex),
    #[error("missing block {0}")]
    MissingBlock(BlockId),
    #[error("missing metadata for block {0}")]
    MissingMetadata(BlockId),
    #[error("missing approvers for block {0}")]
    MissingApprovers(BlockId),
    #[error("storage operation failed due to: {0:?}")]
    Storage(Box<dyn std::error::Error + Send>),
}

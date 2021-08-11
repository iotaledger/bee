// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{prelude::MilestoneIndex, MessageId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("pruning target index {selected} below minimum {minimum}")]
    InvalidTargetIndex {
        selected: MilestoneIndex,
        minimum: MilestoneIndex,
    },
    #[error("missing milestone {0}")]
    MissingMilestone(MilestoneIndex),
    #[error("missing message {0}")]
    MissingMessage(MessageId),
    #[error("missing metadata for message {0}")]
    MissingMetadata(MessageId),
    #[error("missing approvers for message {0}")]
    MissingApprovers(MessageId),
    #[error("storage batch operation failed due to: {0:?}")]
    BatchOperation(Box<dyn std::error::Error + Send>),
    #[error("storage fetch operation failed due to: {0:?}")]
    FetchOperation(Box<dyn std::error::Error + Send>),
}

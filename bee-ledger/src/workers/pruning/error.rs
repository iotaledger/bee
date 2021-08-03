// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{prelude::MilestoneIndex, MessageId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Minimum: {minimum}, Found: {found}")]
    InvalidTargetIndex {
        minimum: MilestoneIndex,
        found: MilestoneIndex,
    },

    #[error("{0}")]
    MissingMilestone(MilestoneIndex),

    #[error("{0}")]
    MissingMessage(MessageId),

    #[error("{0}")]
    MissingMetadata(MessageId),

    #[error("{0}")]
    MissingApprovers(MessageId),

    #[error("{0:?}")]
    BatchOperation(Box<dyn std::error::Error + Send>),

    #[error("{0:?}")]
    FetchOperation(Box<dyn std::error::Error + Send>),
}

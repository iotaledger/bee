// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{milestone::MilestoneIndex, payload::transaction::TransactionId};

/// Represents a newly consumed output.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
pub struct ConsumedOutput {
    target: TransactionId,
    milestone_index: MilestoneIndex,
}

impl ConsumedOutput {
    /// Creates a new [`ConsumedOutput`].
    pub fn new(target: TransactionId, milestone_index: MilestoneIndex) -> Self {
        Self {
            target,
            milestone_index,
        }
    }

    /// Returns the target transaction of the [`ConsumedOutput`].
    pub fn target(&self) -> &TransactionId {
        &self.target
    }

    /// Returns the milestone index of the [`ConsumedOutput`].
    pub fn milestone_index(&self) -> MilestoneIndex {
        self.milestone_index
    }
}

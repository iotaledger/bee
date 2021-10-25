// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{milestone::MilestoneIndex, payload::transaction::TransactionId};
use bee_packable::Packable;

/// Represents a newly consumed output.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
pub struct ConsumedOutput {
    target: TransactionId,
    index: MilestoneIndex,
}

impl ConsumedOutput {
    /// Creates a new `ConsumedOutput`.
    pub fn new(target: TransactionId, index: MilestoneIndex) -> Self {
        Self { target, index }
    }

    /// Returns the target transaction of the `ConsumedOutput`.
    pub fn target(&self) -> &TransactionId {
        &self.target
    }

    /// Returns the milestone index of the `ConsumedOutput`.
    pub fn index(&self) -> MilestoneIndex {
        self.index
    }
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::milestone::MilestoneId;
use bee_packable::Packable;

/// Wraps together the identifiers of the milestones that created and consumed treasury outputs.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
pub struct TreasuryDiff {
    created: MilestoneId,
    consumed: MilestoneId,
}

impl TreasuryDiff {
    /// Creates a new `TreasuryDiff`.
    pub fn new(created: MilestoneId, consumed: MilestoneId) -> Self {
        Self { created, consumed }
    }

    /// Returns the id of the milestone that created the treasury output associated to the `TreasuryDiff`.
    pub fn created(&self) -> &MilestoneId {
        &self.created
    }

    /// Returns the id of the milestone that consumed the treasury input associated to the `TreasuryDiff`.
    pub fn consumed(&self) -> &MilestoneId {
        &self.consumed
    }
}

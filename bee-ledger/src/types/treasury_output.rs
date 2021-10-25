// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{output, payload::milestone::MilestoneId};
use bee_packable::Packable;

/// Records the creation of a treasury output.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
pub struct TreasuryOutput {
    inner: output::TreasuryOutput,
    milestone_id: MilestoneId,
}

impl TreasuryOutput {
    /// Creates a new `TreasuryOutput`.
    pub fn new(inner: output::TreasuryOutput, milestone_id: MilestoneId) -> Self {
        Self { inner, milestone_id }
    }

    /// Returns the inner output of a `TreasuryOutput`.
    pub fn inner(&self) -> &output::TreasuryOutput {
        &self.inner
    }

    /// Returns the id of the milestone that created the `TreasuryOutput`.
    pub fn milestone_id(&self) -> &MilestoneId {
        &self.milestone_id
    }
}

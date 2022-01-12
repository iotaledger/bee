// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::milestone::MilestoneIndex;

use derive_more::From;

/// Defines a milestone index until which the output can not be unlocked.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, bee_packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelockMilestoneIndexFeatureBlock(
    // The milestone index starting from which the output can be consumed.
    MilestoneIndex,
);

impl TimelockMilestoneIndexFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of a [`TimelockMilestoneIndexFeatureBlock`].
    pub const KIND: u8 = 3;

    /// Creates a new [`TimelockMilestoneIndexFeatureBlock`].
    #[inline(always)]
    pub fn new(index: MilestoneIndex) -> Self {
        Self(index)
    }

    /// Returns the index.
    #[inline(always)]
    pub fn index(&self) -> MilestoneIndex {
        self.0
    }
}

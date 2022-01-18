// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::milestone::MilestoneIndex;

use derive_more::From;

/// Defines a milestone index and/or unix timestamp until which the output can not be unlocked.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelockFeatureBlock {
    // The milestone index starting from which the output can be consumed.
    index: MilestoneIndex,
    // Unix time, seconds since unix epoch, starting from which the output can be consumed.
    timestamp: u32,
}

impl TimelockFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of a [`TimelockFeatureBlock`].
    pub const KIND: u8 = 3;

    /// Creates a new [`TimelockFeatureBlock`].
    #[inline(always)]
    pub fn new(index: MilestoneIndex, timestamp: u32) -> Self {
        Self { index, timestamp }
    }

    /// Returns the index of a [`TimelockFeatureBlock`].
    #[inline(always)]
    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    /// Returns the timestamp of a [`TimelockFeatureBlock`].
    #[inline(always)]
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

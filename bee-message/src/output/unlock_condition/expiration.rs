// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::milestone::MilestoneIndex;

use derive_more::From;

/// Defines a milestone index and/or unix time until which only the deposit [`Address`](crate::address::Address) is
/// allowed to unlock the output. After the expiration time, only the [`Address`](crate::address::Address) defined in
/// the [`SenderFeatureBlock`](crate::output::feature_block::SenderFeatureBlock) can unlock it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpirationUnlockCondition {
    // Before this milestone index, [`Address`](crate::address::Address) is allowed to unlock the output.
    // After that, only the [`Address`](crate::address::Address) defined in the
    // [`SenderFeatureBlock`](crate::output::feature_block::SenderFeatureBlock) can.
    index: MilestoneIndex,
    // Before this unix time, seconds since unix epoch, [`Address`](crate::address::Address) is allowed to unlock the
    // output. After that, only the [`Address`](crate::address::Address) defined in the
    // [`SenderFeatureBlock`](crate::output::feature_block::SenderFeatureBlock) can.
    timestamp: u32,
}

impl ExpirationUnlockCondition {
    /// The [`UnlockCondition`](crate::output::UnlockCondition) kind of an [`ExpirationUnlockCondition`].
    pub const KIND: u8 = 3;

    /// Creates a new [`ExpirationUnlockCondition`].
    #[inline(always)]
    pub fn new(index: MilestoneIndex, timestamp: u32) -> Self {
        Self { index, timestamp }
    }

    /// Returns the index of a [`ExpirationUnlockCondition`].
    #[inline(always)]
    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    /// Returns the timestamp of a [`ExpirationUnlockCondition`].
    #[inline(always)]
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use derive_more::From;

/// Defines a unix time until which the output can not be unlocked.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelockUnixFeatureBlock(
    // Unix time, seconds since unix epoch, starting from which the output can be consumed.
    u32,
);

impl TimelockUnixFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of a [`TimelockUnixFeatureBlock`].
    pub const KIND: u8 = 4;

    /// Creates a new [`TimelockUnixFeatureBlock`].
    #[inline(always)]
    pub fn new(timestamp: u32) -> Self {
        Self(timestamp)
    }

    /// Returns the timestamp.
    #[inline(always)]
    pub fn timestamp(&self) -> u32 {
        self.0
    }
}

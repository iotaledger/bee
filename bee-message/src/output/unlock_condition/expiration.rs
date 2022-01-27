// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, milestone::MilestoneIndex};

use derive_more::From;

/// Defines a milestone index and/or unix time until which only Address, defined in Address Unlock Condition, is allowed
/// to unlock the output. After the milestone index and/or unix time, only Return Address can unlock it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpirationUnlockCondition {
    // The address that can unlock the expired output.
    return_address: Address,
    // Before this milestone index, [`AddressUnlockCondition`](crate::unlock_condition::AddressUnlockCondition) is
    // allowed to unlock the output.
    // After that, only the return [`Address`](crate::address::Address) can.
    milestone_index: MilestoneIndex,
    // Before this unix time, seconds since unix epoch,
    // [`AddressUnlockCondition`](crate::unlock_condition::AddressUnlockCondition) is allowed to unlock the output.
    // After that, only the return [`Address`](crate::address::Address) can.
    timestamp: u32,
}

impl ExpirationUnlockCondition {
    /// The [`UnlockCondition`](crate::output::UnlockCondition) kind of an [`ExpirationUnlockCondition`].
    pub const KIND: u8 = 3;

    /// Creates a new [`ExpirationUnlockCondition`].
    #[inline(always)]
    pub fn new(return_address: Address, milestone_index: MilestoneIndex, timestamp: u32) -> Self {
        Self {
            return_address,
            milestone_index,
            timestamp,
        }
    }

    /// Returns the return address of a [`ExpirationUnlockCondition`].
    #[inline(always)]
    pub fn return_address(&self) -> &Address {
        &self.return_address
    }

    /// Returns the milestone index of a [`ExpirationUnlockCondition`].
    #[inline(always)]
    pub fn milestone_index(&self) -> MilestoneIndex {
        self.milestone_index
    }

    /// Returns the timestamp of a [`ExpirationUnlockCondition`].
    #[inline(always)]
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

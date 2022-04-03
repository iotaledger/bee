// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use derive_more::From;
use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use crate::{address::Address, milestone::MilestoneIndex, Error};

/// Defines a milestone index and/or unix time until which only Address, defined in Address Unlock Condition, is allowed
/// to unlock the output. After the milestone index and/or unix time, only Return Address can unlock it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From)]
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
    pub fn new(return_address: Address, milestone_index: MilestoneIndex, timestamp: u32) -> Result<Self, Error> {
        verify_milestone_index_timestamp(milestone_index, timestamp)?;

        Ok(Self {
            return_address,
            milestone_index,
            timestamp,
        })
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

    /// Returns the return address if the condition has expired.
    pub fn return_address_expired(&self, milestone_index: MilestoneIndex, timestamp: u32) -> Option<&Address> {
        if *self.milestone_index() != 0 && self.timestamp() != 0 {
            if milestone_index >= self.milestone_index() && timestamp >= self.timestamp() {
                Some(&self.return_address)
            } else {
                None
            }
        } else if *self.milestone_index() != 0 && milestone_index >= self.milestone_index() {
            Some(&self.return_address)
        } else if self.timestamp() != 0 && timestamp >= self.timestamp() {
            Some(&self.return_address)
        } else {
            None
        }
    }
}

impl Packable for ExpirationUnlockCondition {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.return_address.pack(packer)?;
        self.milestone_index.pack(packer)?;
        self.timestamp.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let return_address = Address::unpack::<_, VERIFY>(unpacker)?;
        let milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).infallible()?;
        let timestamp = u32::unpack::<_, VERIFY>(unpacker).infallible()?;

        if VERIFY {
            verify_milestone_index_timestamp(milestone_index, timestamp).map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            return_address,
            milestone_index,
            timestamp,
        })
    }
}

#[inline]
fn verify_milestone_index_timestamp(milestone_index: MilestoneIndex, timestamp: u32) -> Result<(), Error> {
    if *milestone_index == 0 && timestamp == 0 {
        Err(Error::ExpirationUnlockConditionZero)
    } else {
        Ok(())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use crate::{
        address::dto::AddressDto,
        dto::{is_zero, is_zero_milestone},
        milestone::MilestoneIndex,
    };

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ExpirationUnlockConditionDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "returnAddress")]
        pub return_address: AddressDto,
        #[serde(rename = "milestoneIndex")]
        #[serde(skip_serializing_if = "is_zero_milestone", default)]
        pub milestone_index: MilestoneIndex,
        #[serde(rename = "unixTime")]
        #[serde(skip_serializing_if = "is_zero", default)]
        pub timestamp: u32,
    }
}

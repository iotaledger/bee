// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    constant::{DUST_DEPOSIT_MIN, IOTA_SUPPLY},
    Error,
};

use packable::bounded::BoundedU64;

use core::ops::RangeInclusive;

pub(crate) type StorageDepositAmount = BoundedU64<
    { *StorageDepositReturnUnlockCondition::AMOUNT_RANGE.start() },
    { *StorageDepositReturnUnlockCondition::AMOUNT_RANGE.end() },
>;

/// Defines the amount of IOTAs used as storage deposit that have to be returned to the return [`Address`].
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageDepositReturnUnlockCondition {
    // The [`Address`] to return the amount to.
    return_address: Address,
    // Amount of IOTA coins the consuming transaction should deposit to `return_address`.
    #[packable(unpack_error_with = Error::InvalidStorageDepositAmount)]
    amount: StorageDepositAmount,
}

impl StorageDepositReturnUnlockCondition {
    /// The [`UnlockCondition`](crate::output::UnlockCondition) kind of a [`StorageDepositReturnUnlockCondition`].
    pub const KIND: u8 = 1;
    /// Valid amounts for a [`StorageDepositReturnUnlockCondition`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = DUST_DEPOSIT_MIN..=IOTA_SUPPLY;

    /// Creates a new [`StorageDepositReturnUnlockCondition`].
    #[inline(always)]
    pub fn new(return_address: Address, amount: u64) -> Result<Self, Error> {
        Ok(Self {
            return_address,
            amount: amount.try_into().map_err(Error::InvalidStorageDepositAmount)?,
        })
    }

    /// Returns the return address.
    #[inline(always)]
    pub fn return_address(&self) -> &Address {
        &self.return_address
    }

    /// Returns the amount.
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}

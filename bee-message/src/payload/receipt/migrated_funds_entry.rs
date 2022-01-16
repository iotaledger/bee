// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    constant::{DUST_DEPOSIT_MIN, IOTA_SUPPLY},
    payload::receipt::TailTransactionHash,
    Error,
};

use packable::{bounded::BoundedU64, Packable};

use core::ops::RangeInclusive;

pub(crate) type MigratedFundsAmount =
    BoundedU64<{ *MigratedFundsEntry::AMOUNT_RANGE.start() }, { *MigratedFundsEntry::AMOUNT_RANGE.end() }>;

/// Describes funds which were migrated from a legacy network.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MigratedFundsEntry {
    tail_transaction_hash: TailTransactionHash,
    // The target address of the migrated funds.
    address: Address,
    // The migrated amount.
    #[packable(unpack_error_with = Error::InvalidMigratedFundsEntryAmount)]
    amount: MigratedFundsAmount,
}

impl MigratedFundsEntry {
    /// Range of valid amounts for a [`MigratedFundsEntry`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = DUST_DEPOSIT_MIN..=IOTA_SUPPLY;

    /// Creates a new [`MigratedFundsEntry`].
    pub fn new(tail_transaction_hash: TailTransactionHash, address: Address, amount: u64) -> Result<Self, Error> {
        amount
            .try_into()
            .map(|amount| Self {
                tail_transaction_hash,
                address,
                amount,
            })
            .map_err(Error::InvalidMigratedFundsEntryAmount)
    }

    #[inline(always)]
    /// Returns the tail transaction hash of a [`MigratedFundsEntry`].
    pub fn tail_transaction_hash(&self) -> &TailTransactionHash {
        &self.tail_transaction_hash
    }

    /// Returns the address of a [`MigratedFundsEntry`].
    #[inline(always)]
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the amount of a [`MigratedFundsEntry`].
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::RangeInclusive;

use packable::{bounded::BoundedU64, Packable};

use crate::{
    address::Address, constant::TOKEN_SUPPLY, payload::milestone::option::receipt::TailTransactionHash, Error,
};

const MIGRATED_FUNDS_ENTRY_AMOUNT_MIN: u64 = 1_000_000;

pub(crate) type MigratedFundsAmount =
    BoundedU64<{ *MigratedFundsEntry::AMOUNT_RANGE.start() }, { *MigratedFundsEntry::AMOUNT_RANGE.end() }>;

/// Describes funds which were migrated from a legacy network.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub const AMOUNT_RANGE: RangeInclusive<u64> = MIGRATED_FUNDS_ENTRY_AMOUNT_MIN..=TOKEN_SUPPLY;

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

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{address::dto::AddressDto, error::dto::DtoError};

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct MigratedFundsEntryDto {
        #[serde(rename = "tailTransactionHash")]
        pub tail_transaction_hash: String,
        pub address: AddressDto,
        pub deposit: u64,
    }

    impl From<&MigratedFundsEntry> for MigratedFundsEntryDto {
        fn from(value: &MigratedFundsEntry) -> Self {
            MigratedFundsEntryDto {
                tail_transaction_hash: prefix_hex::encode(value.tail_transaction_hash().as_ref()),
                address: value.address().into(),
                deposit: value.amount(),
            }
        }
    }

    impl TryFrom<&MigratedFundsEntryDto> for MigratedFundsEntry {
        type Error = DtoError;

        fn try_from(value: &MigratedFundsEntryDto) -> Result<Self, Self::Error> {
            let tail_transaction_hash = prefix_hex::decode(&value.tail_transaction_hash)
                .map_err(|_| DtoError::InvalidField("tailTransactionHash"))?;
            Ok(MigratedFundsEntry::new(
                TailTransactionHash::new(tail_transaction_hash)?,
                (&value.address).try_into()?,
                value.deposit,
            )?)
        }
    }
}

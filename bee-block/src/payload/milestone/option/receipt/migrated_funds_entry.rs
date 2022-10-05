// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::Packable;

use crate::{
    address::Address, payload::milestone::option::receipt::TailTransactionHash, protocol::ProtocolParameters, Error,
};

/// Describes funds which were migrated from a legacy network.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_visitor = ProtocolParameters)]
pub struct MigratedFundsEntry {
    tail_transaction_hash: TailTransactionHash,
    // The target address of the migrated funds.
    address: Address,
    // The migrated amount.
    #[packable(verify_with = verify_amount_packable)]
    amount: u64,
}

impl MigratedFundsEntry {
    /// Range of valid amounts for a [`MigratedFundsEntry`].
    pub const AMOUNT_MIN: u64 = 1_000_000;

    /// Creates a new [`MigratedFundsEntry`].
    pub fn new(
        tail_transaction_hash: TailTransactionHash,
        address: Address,
        amount: u64,
        token_supply: u64,
    ) -> Result<Self, Error> {
        verify_amount::<true>(&amount, &token_supply)?;

        Ok(Self {
            tail_transaction_hash,
            address,
            amount,
        })
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
        self.amount
    }
}

fn verify_amount<const VERIFY: bool>(amount: &u64, token_supply: &u64) -> Result<(), Error> {
    if VERIFY && (*amount < MigratedFundsEntry::AMOUNT_MIN || amount > token_supply) {
        Err(Error::InvalidMigratedFundsEntryAmount(*amount))
    } else {
        Ok(())
    }
}

fn verify_amount_packable<const VERIFY: bool>(
    amount: &u64,
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    verify_amount::<VERIFY>(amount, &protocol_parameters.token_supply())
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

    impl MigratedFundsEntry {
        pub fn try_from_dto(value: &MigratedFundsEntryDto, token_supply: u64) -> Result<MigratedFundsEntry, DtoError> {
            let tail_transaction_hash = prefix_hex::decode(&value.tail_transaction_hash)
                .map_err(|_| DtoError::InvalidField("tailTransactionHash"))?;

            Ok(MigratedFundsEntry::new(
                TailTransactionHash::new(tail_transaction_hash)?,
                (&value.address).try_into()?,
                value.deposit,
                token_supply,
            )?)
        }

        pub fn try_from_dto_unverified(value: &MigratedFundsEntryDto) -> Result<MigratedFundsEntry, DtoError> {
            let tail_transaction_hash = prefix_hex::decode(&value.tail_transaction_hash)
                .map_err(|_| DtoError::InvalidField("tailTransactionHash"))?;

            Ok(Self {
                tail_transaction_hash: TailTransactionHash::new(tail_transaction_hash)?,
                address: (&value.address).try_into()?,
                amount: value.deposit,
            })
        }
    }
}

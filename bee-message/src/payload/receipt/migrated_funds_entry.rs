// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants::IOTA_SUPPLY,
    output::{SignatureLockedSingleOutput, DUST_THRESHOLD},
    payload::receipt::TailTransactionHash,
    Error,
};

use bee_packable::{error::UnpackError, packer::Packer, unpacker::Unpacker, Packable};

use core::ops::RangeInclusive;
use std::convert::Infallible;

/// Range of valid amounts for migrated funds entries.
pub const VALID_MIGRATED_FUNDS_ENTRY_AMOUNTS: RangeInclusive<u64> = DUST_THRESHOLD..=IOTA_SUPPLY;

/// Describes funds which were migrated from a legacy network.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MigratedFundsEntry {
    tail_transaction_hash: TailTransactionHash,
    output: SignatureLockedSingleOutput,
}

impl MigratedFundsEntry {
    /// Creates a new `MigratedFundsEntry`.
    pub fn new(tail_transaction_hash: TailTransactionHash, output: SignatureLockedSingleOutput) -> Result<Self, Error> {
        if !VALID_MIGRATED_FUNDS_ENTRY_AMOUNTS.contains(&output.amount()) {
            return Err(Error::InvalidMigratedFundsEntryAmount(output.amount()));
        }

        Ok(Self {
            tail_transaction_hash,
            output,
        })
    }

    /// Returns the tail transaction hash of a `MigratedFundsEntry`.
    pub fn tail_transaction_hash(&self) -> &TailTransactionHash {
        &self.tail_transaction_hash
    }

    /// Returns the output of a `MigratedFundsEntry`.
    pub fn output(&self) -> &SignatureLockedSingleOutput {
        &self.output
    }
}

impl Packable for MigratedFundsEntry {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.tail_transaction_hash.pack(packer)?;
        self.output.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let tail_transaction_hash = TailTransactionHash::unpack::<_, VERIFY>(unpacker)?;
        let output = SignatureLockedSingleOutput::unpack::<_, VERIFY>(unpacker)?;

        Self::new(tail_transaction_hash, output).map_err(UnpackError::Packable)
    }
}

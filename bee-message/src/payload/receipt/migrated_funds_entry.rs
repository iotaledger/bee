// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants::IOTA_SUPPLY, output::SignatureLockedSingleOutput, payload::receipt::TailTransactionHash, Error,
};

use bee_common::packable::{Packable, Read, Write};

use core::ops::RangeInclusive;

/// Range of allowed amount for migrated funds entries.
pub const MIGRATED_FUNDS_ENTRY_AMOUNT: RangeInclusive<u64> = 1_000_000..=IOTA_SUPPLY;

/// Describes funds which were migrated from a legacy network.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MigratedFundsEntry {
    tail_transaction_hash: TailTransactionHash,
    output: SignatureLockedSingleOutput,
}

impl MigratedFundsEntry {
    /// Creates a new `MigratedFundsEntry`.
    pub fn new(tail_transaction_hash: TailTransactionHash, output: SignatureLockedSingleOutput) -> Result<Self, Error> {
        if !MIGRATED_FUNDS_ENTRY_AMOUNT.contains(&output.amount()) {
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
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.tail_transaction_hash.packed_len() + self.output.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.tail_transaction_hash.pack(writer)?;
        self.output.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let tail_transaction_hash = TailTransactionHash::unpack_inner::<R, CHECK>(reader)?;
        let output = SignatureLockedSingleOutput::unpack_inner::<R, CHECK>(reader)?;

        Self::new(tail_transaction_hash, output)
    }
}

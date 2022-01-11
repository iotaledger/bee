// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constant::{DUST_DEPOSIT_MIN, IOTA_SUPPLY},
    output::SimpleOutput,
    payload::receipt::TailTransactionHash,
    Error,
};

use bee_packable::{error::UnpackError, packer::Packer, unpacker::Unpacker, Packable};

use core::ops::RangeInclusive;

/// Describes funds which were migrated from a legacy network.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MigratedFundsEntry {
    tail_transaction_hash: TailTransactionHash,
    output: SimpleOutput,
}

impl MigratedFundsEntry {
    /// Range of valid amounts for a [`MigratedFundsEntry`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = DUST_DEPOSIT_MIN..=IOTA_SUPPLY;

    /// Creates a new [`MigratedFundsEntry`].
    pub fn new(tail_transaction_hash: TailTransactionHash, output: SimpleOutput) -> Result<Self, Error> {
        if !MigratedFundsEntry::AMOUNT_RANGE.contains(&output.amount()) {
            return Err(Error::InvalidMigratedFundsEntryAmount(output.amount()));
        }

        Ok(Self {
            tail_transaction_hash,
            output,
        })
    }

    /// Returns the tail transaction hash of a [`MigratedFundsEntry`].
    pub fn tail_transaction_hash(&self) -> &TailTransactionHash {
        &self.tail_transaction_hash
    }

    /// Returns the output of a [`MigratedFundsEntry`].
    pub fn output(&self) -> &SimpleOutput {
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
        let output = SimpleOutput::unpack::<_, VERIFY>(unpacker)?;

        Self::new(tail_transaction_hash, output).map_err(UnpackError::Packable)
    }
}

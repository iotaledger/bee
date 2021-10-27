// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the receipt payload.

mod migrated_funds_entry;
mod tail_transaction_hash;

pub use migrated_funds_entry::{MigratedFundsEntry, VALID_MIGRATED_FUNDS_ENTRY_AMOUNTS};
pub use tail_transaction_hash::{TailTransactionHash, TAIL_TRANSACTION_HASH_LEN};

use crate::{
    constants::INPUT_OUTPUT_COUNT_RANGE,
    milestone::MilestoneIndex,
    payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, Payload},
    Error,
};

use bee_common::{
    ord::is_unique_sorted,
    packable::{Packable, Read, Write},
};

use core::ops::RangeInclusive;
use std::collections::HashMap;

const MIGRATED_FUNDS_ENTRY_RANGE: RangeInclusive<usize> = INPUT_OUTPUT_COUNT_RANGE;

/// Receipt is a listing of migrated funds.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ReceiptPayload {
    migrated_at: MilestoneIndex,
    last: bool,
    funds: Vec<MigratedFundsEntry>,
    transaction: Payload,
}

impl ReceiptPayload {
    /// The payload kind of a `ReceiptPayload`.
    pub const KIND: u32 = 3;

    /// Creates a new `ReceiptPayload`.
    pub fn new(
        migrated_at: MilestoneIndex,
        last: bool,
        funds: Vec<MigratedFundsEntry>,
        transaction: Payload,
    ) -> Result<Self, Error> {
        if !MIGRATED_FUNDS_ENTRY_RANGE.contains(&funds.len()) {
            return Err(Error::InvalidReceiptFundsCount(funds.len()));
        }

        if !matches!(transaction, Payload::TreasuryTransaction(_)) {
            return Err(Error::InvalidPayloadKind(transaction.kind()));
        }

        // Funds must be lexicographically sorted and unique in their serialised forms.
        if !is_unique_sorted(funds.iter().map(Packable::pack_new)) {
            return Err(Error::ReceiptFundsNotUniqueSorted);
        }

        let mut tail_transaction_hashes = HashMap::with_capacity(funds.len());
        for (index, funds) in funds.iter().enumerate() {
            if let Some(previous) = tail_transaction_hashes.insert(funds.tail_transaction_hash().as_ref(), index) {
                return Err(Error::TailTransactionHashNotUnique(previous, index));
            }
        }

        Ok(Self {
            migrated_at,
            last,
            funds,
            transaction,
        })
    }

    /// Returns the milestone index at which the funds of a `ReceiptPayload` were migrated at in the legacy network.
    pub fn migrated_at(&self) -> MilestoneIndex {
        self.migrated_at
    }

    /// Returns whether a `ReceiptPayload` is the final one for a given migrated at index.
    pub fn last(&self) -> bool {
        self.last
    }

    /// The funds which were migrated with a `ReceiptPayload`.
    pub fn funds(&self) -> &[MigratedFundsEntry] {
        &self.funds
    }

    /// The `TreasuryTransaction` used to fund the funds of a `ReceiptPayload`.
    pub fn transaction(&self) -> &Payload {
        &self.transaction
    }

    /// Returns the sum of all `MigratedFundsEntry` items within a `ReceiptPayload`.
    pub fn amount(&self) -> u64 {
        self.funds.iter().fold(0, |acc, funds| acc + funds.output().amount())
    }
}

impl Packable for ReceiptPayload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.migrated_at.packed_len()
            + self.last.packed_len()
            + 0u16.packed_len()
            + self.funds.iter().map(Packable::packed_len).sum::<usize>()
            + option_payload_packed_len(Some(&self.transaction))
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.migrated_at.pack(writer)?;
        self.last.pack(writer)?;
        (self.funds.len() as u16).pack(writer)?;
        for fund in self.funds.iter() {
            fund.pack(writer)?;
        }
        option_payload_pack(writer, Some(&self.transaction))?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let migrated_at = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;
        let last = bool::unpack_inner::<R, CHECK>(reader)?;
        let funds_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut funds = Vec::with_capacity(funds_len);
        for _ in 0..funds_len {
            funds.push(MigratedFundsEntry::unpack_inner::<R, CHECK>(reader)?);
        }
        let transaction = option_payload_unpack::<R, CHECK>(reader)?
            .1
            .ok_or(Self::Error::MissingPayload)?;

        Self::new(migrated_at, last, funds, transaction)
    }
}

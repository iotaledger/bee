// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the receipt payload.

mod migrated_funds_entry;
mod tail_transaction_hash;

pub use migrated_funds_entry::MigratedFundsEntry;
pub use tail_transaction_hash::TailTransactionHash;

use crate::{
    milestone::MilestoneIndex,
    output::OUTPUT_COUNT_RANGE,
    payload::{OptionalPayload, Payload},
    Error,
};

use bee_common::ord::is_unique_sorted;
use bee_packable::{
    bounded::BoundedU16,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::VecPrefix,
    unpacker::Unpacker,
    PackableExt,
};

use core::ops::RangeInclusive;
use std::collections::HashMap;

const MIGRATED_FUNDS_ENTRY_RANGE: RangeInclusive<u16> = OUTPUT_COUNT_RANGE;

pub(crate) type ReceiptFundsCount =
    BoundedU16<{ *MIGRATED_FUNDS_ENTRY_RANGE.start() }, { *MIGRATED_FUNDS_ENTRY_RANGE.end() }>;

/// Receipt is a listing of migrated funds.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ReceiptPayload {
    migrated_at: MilestoneIndex,
    last: bool,
    funds: VecPrefix<MigratedFundsEntry, ReceiptFundsCount>,
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
        let funds = VecPrefix::<MigratedFundsEntry, ReceiptFundsCount>::try_from(funds)
            .map_err(Error::InvalidReceiptFundsCount)?;

        Self::from_vec_prefix(migrated_at, last, funds, transaction)
    }

    fn from_vec_prefix(
        migrated_at: MilestoneIndex,
        last: bool,
        funds: VecPrefix<MigratedFundsEntry, ReceiptFundsCount>,
        transaction: Payload,
    ) -> Result<Self, Error> {
        if !matches!(transaction, Payload::TreasuryTransaction(_)) {
            return Err(Error::InvalidPayloadKind(transaction.kind()));
        }

        // Funds must be lexicographically sorted and unique in their serialised forms.
        if !is_unique_sorted(funds.iter().map(PackableExt::pack_to_vec)) {
            return Err(Error::ReceiptFundsNotUniqueSorted);
        }

        let mut tail_transaction_hashes = HashMap::with_capacity(funds.len());
        for (index, funds) in funds.iter().enumerate() {
            if let Some(previous) = tail_transaction_hashes.insert(funds.tail_transaction_hash().as_ref(), index) {
                return Err(Error::TailTransactionHashNotUnique {
                    previous,
                    current: index,
                });
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

impl bee_packable::Packable for ReceiptPayload {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.migrated_at.pack(packer)?;
        self.last.pack(packer)?;
        self.funds.pack(packer)?;
        OptionalPayload::pack_ref(&self.transaction, packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let migrated_at = MilestoneIndex::unpack::<_, VERIFY>(unpacker).infallible()?;
        let last = bool::unpack::<_, VERIFY>(unpacker).infallible()?;
        let funds = VecPrefix::<MigratedFundsEntry, ReceiptFundsCount>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| {
                err.unwrap_packable_or_else(|prefix_err| Error::InvalidReceiptFundsCount(prefix_err.into()))
            })?;
        let transaction = Into::<Option<Payload>>::into(OptionalPayload::unpack::<_, VERIFY>(unpacker)?)
            .ok_or(UnpackError::Packable(Error::MissingPayload))?;

        Self::from_vec_prefix(migrated_at, last, funds, transaction).map_err(UnpackError::Packable)
    }
}

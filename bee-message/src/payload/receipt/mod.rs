// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the receipt payload.

mod migrated_funds_entry;
mod tail_transaction_hash;

pub(crate) use migrated_funds_entry::MigratedFundsAmount;
pub use migrated_funds_entry::MigratedFundsEntry;
pub use tail_transaction_hash::TailTransactionHash;

use crate::{milestone::MilestoneIndex, output::OUTPUT_COUNT_RANGE, payload::Payload, Error};

use hashbrown::HashMap;
use iterator_sorted::is_unique_sorted;
use packable::{bounded::BoundedU16, prefix::VecPrefix, Packable, PackableExt};

use alloc::vec::Vec;
use core::ops::RangeInclusive;

const MIGRATED_FUNDS_ENTRY_RANGE: RangeInclusive<u16> = OUTPUT_COUNT_RANGE;

pub(crate) type ReceiptFundsCount =
    BoundedU16<{ *MIGRATED_FUNDS_ENTRY_RANGE.start() }, { *MIGRATED_FUNDS_ENTRY_RANGE.end() }>;

/// Receipt is a listing of migrated funds.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct ReceiptPayload {
    migrated_at: MilestoneIndex,
    last: bool,
    #[packable(unpack_error_with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidReceiptFundsCount(p.into())))]
    #[packable(verify_with = verify_funds)]
    funds: VecPrefix<MigratedFundsEntry, ReceiptFundsCount>,
    #[packable(verify_with = verify_treasury_transaction)]
    treasury_transaction: Payload,
}

impl ReceiptPayload {
    /// The payload kind of a [`ReceiptPayload`].
    pub const KIND: u32 = 3;

    /// Creates a new [`ReceiptPayload`].
    pub fn new(
        migrated_at: MilestoneIndex,
        last: bool,
        funds: Vec<MigratedFundsEntry>,
        treasury_transaction: Payload,
    ) -> Result<Self, Error> {
        let funds = VecPrefix::<MigratedFundsEntry, ReceiptFundsCount>::try_from(funds)
            .map_err(Error::InvalidReceiptFundsCount)?;

        verify_treasury_transaction::<true>(&treasury_transaction)?;
        verify_funds::<true>(&funds)?;

        Ok(Self {
            migrated_at,
            last,
            funds,
            treasury_transaction,
        })
    }

    /// Returns the milestone index at which the funds of a [`ReceiptPayload`] were migrated at in the legacy network.
    pub fn migrated_at(&self) -> MilestoneIndex {
        self.migrated_at
    }

    /// Returns whether a [`ReceiptPayload`] is the final one for a given migrated at index.
    pub fn last(&self) -> bool {
        self.last
    }

    /// The funds which were migrated with a [`ReceiptPayload`].
    pub fn funds(&self) -> &[MigratedFundsEntry] {
        &self.funds
    }

    /// The [`TreasuryTransaction`] used to fund the funds of a [`ReceiptPayload`].
    pub fn treasury_transaction(&self) -> &Payload {
        &self.treasury_transaction
    }

    /// Returns the sum of all [`MigratedFundsEntry`] items within a [`ReceiptPayload`].
    pub fn amount(&self) -> u64 {
        self.funds.iter().fold(0, |acc, funds| acc + funds.amount())
    }
}

fn verify_funds<const VERIFY: bool>(funds: &[MigratedFundsEntry]) -> Result<(), Error> {
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

    Ok(())
}

fn verify_treasury_transaction<const VERIFY: bool>(treasury_transaction: &Payload) -> Result<(), Error> {
    if !matches!(treasury_transaction, Payload::TreasuryTransaction(_)) {
        Err(Error::InvalidPayloadKind(treasury_transaction.kind()))
    } else {
        Ok(())
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod migrated_funds_entry;

pub use migrated_funds_entry::{MigratedFundsEntry, MIGRATED_FUNDS_ENTRY_AMOUNT};

use crate::{payload::Payload, Error};

use bee_common::packable::{Packable, Read, Write};

use core::ops::RangeInclusive;

pub(crate) const RECEIPT_PAYLOAD_KIND: u32 = 3;
const MIGRATED_FUNDS_ENTRY_RANGE: RangeInclusive<usize> = 1..=127;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReceiptPayload {
    index: u32,
    last: bool,
    funds: Vec<MigratedFundsEntry>,
    transaction: Payload,
}

impl ReceiptPayload {
    pub fn new(index: u32, last: bool, funds: Vec<MigratedFundsEntry>, transaction: Payload) -> Result<Self, Error> {
        if !MIGRATED_FUNDS_ENTRY_RANGE.contains(&funds.len()) {
            return Err(Error::InvalidReceiptFundsCount(funds.len()));
        }

        // TODO check lexicographic order and uniqueness of funds.

        if !matches!(transaction, Payload::TreasuryTransaction(_)) {
            return Err(Error::InvalidPayloadKind(transaction.kind()));
        }

        Ok(Self {
            index,
            last,
            funds,
            transaction,
        })
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn last(&self) -> bool {
        self.last
    }

    pub fn funds(&self) -> &[MigratedFundsEntry] {
        &self.funds
    }

    pub fn transaction(&self) -> &Payload {
        &self.transaction
    }

    pub fn amount(&self) -> u64 {
        self.funds.iter().fold(0, |acc, funds| acc + funds.output().amount())
    }
}

impl Packable for ReceiptPayload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.index.packed_len()
            + self.last.packed_len()
            + 0u8.packed_len()
            + self.funds.iter().map(Packable::packed_len).sum::<usize>()
            + self.transaction.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;
        self.last.pack(writer)?;
        (self.funds.len() as u8).pack(writer)?;
        for fund in self.funds.iter() {
            fund.pack(writer)?;
        }
        self.transaction.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = u32::unpack(reader)?;
        let last = bool::unpack(reader)?;
        let funds_len = u8::unpack(reader)? as usize;
        let mut funds = Vec::with_capacity(funds_len);
        for _ in 0..funds_len {
            funds.push(MigratedFundsEntry::unpack(reader)?);
        }
        let transaction = Payload::unpack(reader)?;

        Self::new(index, last, funds, transaction)
    }
}

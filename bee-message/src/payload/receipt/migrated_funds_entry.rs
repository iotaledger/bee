// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constants::IOTA_SUPPLY, Error};

use bee_common::packable::{Packable, Read, Write};

use core::{convert::TryInto, ops::RangeInclusive};

const MIGRATED_FUNDS_ENTRY_AMOUNT: RangeInclusive<u64> = 1_000_000..=IOTA_SUPPLY;
const TAIL_TRANSACTION_HASH_LEN: usize = 49;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MigratedFundsEntry {
    // TODO switch to array when const generics serde is available
    tail_transaction_hash: Box<[u8]>,
    address: Address,
    amount: u64,
}

impl MigratedFundsEntry {
    pub fn new(tail_transaction_hash: [u8; 49], address: Address, amount: u64) -> Result<Self, Error> {
        if !MIGRATED_FUNDS_ENTRY_AMOUNT.contains(&amount) {
            return Err(Error::InvalidMigratedFundsEntryAmount(amount));
        }

        Ok(Self {
            tail_transaction_hash: Box::new(tail_transaction_hash),
            address,
            amount,
        })
    }

    pub fn tail_transaction_hash(&self) -> &[u8; TAIL_TRANSACTION_HASH_LEN] {
        self.tail_transaction_hash.as_ref().try_into().unwrap()
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl Packable for MigratedFundsEntry {
    type Error = Error;

    fn packed_len(&self) -> usize {
        TAIL_TRANSACTION_HASH_LEN + self.address.packed_len() + self.amount.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write_all(&self.tail_transaction_hash)?;
        self.address.pack(writer)?;
        self.amount.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut tail_transaction_hash = [0u8; TAIL_TRANSACTION_HASH_LEN];
        reader.read_exact(&mut tail_transaction_hash)?;

        Self::new(tail_transaction_hash, Address::unpack(reader)?, u64::unpack(reader)?)
    }
}

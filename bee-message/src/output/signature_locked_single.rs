// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constants::IOTA_SUPPLY, Error};

use bee_common::packable::{Packable, Read, Write};

use core::ops::RangeInclusive;

const SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureLockedSingleOutput {
    address: Address,
    amount: u64,
}

impl SignatureLockedSingleOutput {
    pub const KIND: u8 = 0;

    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        if !SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT.contains(&amount) {
            return Err(Error::InvalidAmount(amount));
        }

        Ok(Self { address, amount })
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl Packable for SignatureLockedSingleOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.address.packed_len() + self.amount.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.address.pack(writer)?;
        self.amount.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let address = Address::unpack_inner::<R, CHECK>(reader)?;
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;

        Self::new(address, amount)
    }
}

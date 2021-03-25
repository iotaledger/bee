// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, constants::IOTA_SUPPLY, Error};

use bee_common::packable::{Packable, Read, Write};

use core::ops::RangeInclusive;

const SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_AMOUNT: RangeInclusive<u64> = 1_000_000..=IOTA_SUPPLY;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureLockedDustAllowanceOutput {
    address: Address,
    amount: u64,
}

impl SignatureLockedDustAllowanceOutput {
    pub const KIND: u8 = 1;

    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        if !SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_AMOUNT.contains(&amount) {
            return Err(Error::InvalidDustAllowanceAmount(amount));
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

impl Packable for SignatureLockedDustAllowanceOutput {
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

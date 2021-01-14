// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::transaction::{constants::IOTA_SUPPLY, Address},
    Error,
};

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

pub(crate) const SIGNATURE_LOCKED_SINGLE_TYPE: u8 = 0;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Ord, PartialOrd)]
pub struct SignatureLockedSingleOutput {
    address: Address,
    amount: u64,
}

impl SignatureLockedSingleOutput {
    pub fn new(address: Address, amount: u64) -> Result<Self, Error> {
        if amount == 0 || amount > IOTA_SUPPLY {
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::new(Address::unpack(reader)?, u64::unpack(reader)?)
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::transaction::Address, Error};

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use core::num::NonZeroU64;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Ord, PartialOrd)]
pub struct SignatureLockedSingleOutput {
    address: Address,
    amount: NonZeroU64,
}

impl SignatureLockedSingleOutput {
    pub fn new(address: Address, amount: NonZeroU64) -> Self {
        Self { address, amount }
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn amount(&self) -> NonZeroU64 {
        self.amount
    }
}

impl Packable for SignatureLockedSingleOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.address.packed_len() + u64::from(self.amount).packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.address.pack(writer)?;
        u64::from(self.amount).pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let address = Address::unpack(reader)?;
        // TODO unwrap
        let amount = NonZeroU64::new(u64::unpack(reader)?).unwrap();

        Ok(Self { address, amount })
    }
}

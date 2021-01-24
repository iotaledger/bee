// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::Error;

use bee_common::packable::{Packable, Read, Write};

#[derive(Debug)]
pub struct Balance {
    pub(crate) balance: u64,
    pub(crate) dust_allowance: u64,
    pub(crate) dust_output: u64,
}

impl Balance {
    pub fn new(balance: u64, dust_allowance: u64, dust_output: u64) -> Self {
        Self {
            balance,
            dust_allowance,
            dust_output,
        }
    }
}

impl Packable for Balance {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.balance.packed_len() + self.dust_allowance.packed_len() + self.dust_output.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.balance.pack(writer)?;
        self.dust_allowance.pack(writer)?;
        self.dust_output.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Balance::new(
            u64::unpack(reader)?,
            u64::unpack(reader)?,
            u64::unpack(reader)?,
        ))
    }
}

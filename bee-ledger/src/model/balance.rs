// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::Error;

use bee_common::packable::{Packable, Read, Write};

#[derive(Debug)]
pub struct Balance {
    balance: u64,
    dust_allowance: u64,
    output_count: u64,
}

impl Balance {
    pub fn new(balance: u64, dust_allowance: u64, output_count: u64) -> Self {
        Self {
            balance,
            dust_allowance,
            output_count,
        }
    }
}

impl Packable for Balance {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.balance.packed_len() + self.dust_allowance.packed_len() + self.output_count.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.balance.pack(writer)?;
        self.dust_allowance.pack(writer)?;
        self.output_count.pack(writer)?;

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

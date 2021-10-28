// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct FoundryOutput {
    amount: u64,
}

impl FoundryOutput {
    /// The output kind of a `FoundryOutput`.
    pub const KIND: u8 = 4;

    /// Creates a new `FoundryOutput`.
    pub fn new(amount: u64) -> Self {
        Self { amount }
    }

    /// Returns the amount of a `FoundryOutput`.
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl Packable for FoundryOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(u64::unpack_inner::<R, CHECK>(reader)?))
    }
}

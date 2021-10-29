// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

///
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize, derive_more::From))]
pub struct NftUnlock(u16);

impl NftUnlock {
    /// The unlock kind of a `NftUnlock`.
    pub const KIND: u8 = 3;

    /// Creates a new `NftUnlock`.
    pub fn new(index: u16) -> Self {
        Self(index)
    }

    /// Return the index of a `NftUnlock`.
    pub fn index(&self) -> u16 {
        self.0
    }
}

impl Packable for NftUnlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(u16::unpack_inner::<R, CHECK>(reader)?))
    }
}

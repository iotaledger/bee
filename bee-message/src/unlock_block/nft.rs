// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

/// Points to the unlock block of a consumed NFT output.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize, derive_more::From))]
pub struct NftUnlockBlock {
    /// Index of input and unlock block corresponding to an [`NftOutput`](crate::output::NftOutput).
    index: u16,
}

impl NftUnlockBlock {
    /// The [`UnlockBlock`](crate::unlock_block::UnlockBlock) kind of a [`NftUnlockBlock`].
    pub const KIND: u8 = 3;

    /// Creates a new [`NftUnlockBlock`].
    pub fn new(index: u16) -> Self {
        Self(index)
    }

    /// Return the index of a [`NftUnlockBlock`].
    pub fn index(&self) -> u16 {
        self.0
    }
}

impl Packable for NftUnlockBlock {
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

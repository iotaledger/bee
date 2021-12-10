// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{unlock_block::UNLOCK_BLOCK_INDEX_RANGE, Error};

use bee_common::packable::{Packable, Read, Write};

/// Points to the unlock block of a consumed alias output.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct AliasUnlockBlock(
    // Index of input and unlock block corresponding to an [`AliasOutput`].
    u16,
);

impl TryFrom<u16> for AliasUnlockBlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        validate_index(index)?;

        Ok(Self(index))
    }
}

impl AliasUnlockBlock {
    /// The [`UnlockBlock`](crate::unlock_block::UnlockBlock) kind of an [`AliasUnlockBlock`].
    pub const KIND: u8 = 2;

    /// Creates a new [`AliasUnlockBlock`].
    pub fn new(index: u16) -> Result<Self, Error> {
        index.try_into()
    }

    /// Return the index of an [`AliasUnlockBlock`].
    pub fn index(&self) -> u16 {
        self.0
    }
}

impl Packable for AliasUnlockBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = u16::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_index(index)?;
        }

        Ok(Self(index))
    }
}

#[inline]
fn validate_index(index: u16) -> Result<(), Error> {
    if !UNLOCK_BLOCK_INDEX_RANGE.contains(&index) {
        return Err(Error::InvalidAliasIndex(index));
    }

    Ok(())
}

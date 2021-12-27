// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{unlock_block::UNLOCK_BLOCK_INDEX_RANGE, Error};

use bee_common::packable::{Packable as OldPackable, Read, Write};
use bee_packable::bounded::BoundedU16;

pub(crate) type ReferenceIndex = BoundedU16<{ *UNLOCK_BLOCK_INDEX_RANGE.start() }, { *UNLOCK_BLOCK_INDEX_RANGE.end() }>;

/// An [`UnlockBlock`](crate::unlock_block::UnlockBlock) that refers to another unlock block.
#[derive(Clone, Debug, Eq, PartialEq, Hash, bee_packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidReferenceIndex)]
pub struct ReferenceUnlockBlock(ReferenceIndex);

impl TryFrom<u16> for ReferenceUnlockBlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        index.try_into().map(Self).map_err(Error::InvalidReferenceIndex)
    }
}

impl ReferenceUnlockBlock {
    /// The [`UnlockBlock`](crate::unlock_block::UnlockBlock) kind of a [`ReferenceUnlockBlock`].
    pub const KIND: u8 = 1;

    /// Creates a new [`ReferenceUnlockBlock`].
    #[inline(always)]
    pub fn new(index: u16) -> Result<Self, Error> {
        Self::try_from(index)
    }

    /// Return the index of a [`ReferenceUnlockBlock`].
    #[inline(always)]
    pub fn index(&self) -> u16 {
        self.0.get()
    }
}

impl OldPackable for ReferenceUnlockBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index().pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = u16::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_index(index)?;
        }

        index.try_into()
    }
}

#[inline]
fn validate_index(index: u16) -> Result<(), Error> {
    ReferenceUnlockBlock::try_from(index)?;

    Ok(())
}

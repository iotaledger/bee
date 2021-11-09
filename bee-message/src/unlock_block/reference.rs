// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{unlock_block::UNLOCK_BLOCK_INDEX_RANGE, Error};

use bee_common::packable::{Packable, Read, Write};

/// An [`UnlockBlock`](crate::unlock::UnlockBlock) that refers to another unlock block.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ReferenceUnlockBlock(u16);

impl ReferenceUnlockBlock {
    /// The unlock kind of a `ReferenceUnlockBlock`.
    pub const KIND: u8 = 1;

    /// Creates a new `ReferenceUnlockBlock`.
    pub fn new(index: u16) -> Result<Self, Error> {
        if !UNLOCK_BLOCK_INDEX_RANGE.contains(&index) {
            return Err(Error::InvalidReferenceIndex(index));
        }

        Ok(Self(index))
    }

    /// Return the index of a `ReferenceUnlockBlock`.
    pub fn index(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for ReferenceUnlockBlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        Self::new(index)
    }
}

impl Packable for ReferenceUnlockBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Self::new(u16::unpack_inner::<R, CHECK>(reader)?)
    }
}

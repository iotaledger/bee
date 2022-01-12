// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{unlock_block::UNLOCK_BLOCK_INDEX_RANGE, Error};

use bee_packable::bounded::BoundedU16;

pub(crate) type AliasIndex = BoundedU16<{ *UNLOCK_BLOCK_INDEX_RANGE.start() }, { *UNLOCK_BLOCK_INDEX_RANGE.end() }>;

/// Points to the unlock block of a consumed alias output.
#[derive(Clone, Debug, Eq, PartialEq, Hash, bee_packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidAliasIndex)]
pub struct AliasUnlockBlock(
    // Index of input and unlock block corresponding to an [`AliasOutput`].
    AliasIndex,
);

impl TryFrom<u16> for AliasUnlockBlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        index.try_into().map(Self).map_err(Error::InvalidAliasIndex)
    }
}

impl AliasUnlockBlock {
    /// The [`UnlockBlock`](crate::unlock_block::UnlockBlock) kind of an [`AliasUnlockBlock`].
    pub const KIND: u8 = 2;

    /// Creates a new [`AliasUnlockBlock`].
    #[inline(always)]
    pub fn new(index: u16) -> Result<Self, Error> {
        Self::try_from(index)
    }

    /// Return the index of an [`AliasUnlockBlock`].
    #[inline(always)]
    pub fn index(&self) -> u16 {
        self.0.get()
    }
}

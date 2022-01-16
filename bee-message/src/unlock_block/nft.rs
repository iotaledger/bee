// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{unlock_block::UNLOCK_BLOCK_INDEX_RANGE, Error};

use packable::bounded::BoundedU16;

pub(crate) type NftIndex = BoundedU16<{ *UNLOCK_BLOCK_INDEX_RANGE.start() }, { *UNLOCK_BLOCK_INDEX_RANGE.end() }>;

/// Points to the unlock block of a consumed NFT output.
#[derive(Clone, Debug, Eq, PartialEq, Hash, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidNftIndex)]
pub struct NftUnlockBlock(
    /// Index of input and unlock block corresponding to an [`NftOutput`](crate::output::NftOutput).
    NftIndex,
);

impl TryFrom<u16> for NftUnlockBlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        index.try_into().map(Self).map_err(Error::InvalidNftIndex)
    }
}

impl NftUnlockBlock {
    /// The [`UnlockBlock`](crate::unlock_block::UnlockBlock) kind of a [`NftUnlockBlock`].
    pub const KIND: u8 = 3;

    /// Creates a new [`NftUnlockBlock`].
    #[inline(always)]
    pub fn new(index: u16) -> Result<Self, Error> {
        Self::try_from(index)
    }

    /// Return the index of a [`NftUnlockBlock`].
    #[inline(always)]
    pub fn index(&self) -> u16 {
        self.0.get()
    }
}

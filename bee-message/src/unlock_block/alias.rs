// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{unlock_block::UnlockBlockIndex, Error};

/// Points to the unlock block of a consumed alias output.
#[derive(Clone, Debug, Eq, PartialEq, Hash, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidAliasIndex)]
pub struct AliasUnlockBlock(
    /// Index of input and unlock block corresponding to an [`AliasOutput`](crate::output::AliasOutput).
    UnlockBlockIndex,
);

impl TryFrom<u16> for AliasUnlockBlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        Self::new(index)
    }
}

impl AliasUnlockBlock {
    /// The [`UnlockBlock`](crate::unlock_block::UnlockBlock) kind of an [`AliasUnlockBlock`].
    pub const KIND: u8 = 2;

    /// Creates a new [`AliasUnlockBlock`].
    #[inline(always)]
    pub fn new(index: u16) -> Result<Self, Error> {
        index.try_into().map(Self).map_err(Error::InvalidAliasIndex)
    }

    /// Return the index of an [`AliasUnlockBlock`].
    #[inline(always)]
    pub fn index(&self) -> u16 {
        self.0.get()
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    /// Points to the unlock block of a consumed alias output.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct AliasUnlockBlockDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "reference")]
        pub index: u16,
    }
}

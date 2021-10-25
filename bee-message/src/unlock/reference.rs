// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{constants::INPUT_OUTPUT_INDEX_MAX, Error};

use bee_packable::{
    bounded::BoundedU16,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::convert::Infallible;

/// An [`UnlockBlock`](crate::unlock::UnlockBlock) that refers to another unlock block.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidReferenceIndex)]
pub struct ReferenceUnlock(BoundedU16<0, INPUT_OUTPUT_INDEX_MAX>);

impl ReferenceUnlock {
    /// The unlock kind of a `ReferenceUnlock`.
    pub const KIND: u8 = 1;

    /// Creates a new `ReferenceUnlock`.
    pub fn new(index: u16) -> Result<Self, Error> {
        Ok(Self(index.try_into().map_err(Error::InvalidReferenceIndex)?))
    }

    /// Return the index of a `ReferenceUnlock`.
    pub fn index(&self) -> u16 {
        self.0.into()
    }
}

impl TryFrom<u16> for ReferenceUnlock {
    type Error = Error;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        Self::new(index)
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, unlock::UNLOCK_BLOCK_INDEX_RANGE, MessageUnpackError};

use bee_packable::{coerce::*, PackError, Packable, Packer, UnpackError, Unpacker};

use core::{
    convert::{Infallible, TryFrom},
    fmt,
};

/// Error encountered unpacking a [`ReferenceUnlock`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum ReferenceUnlockUnpackError {
    ValidationError(ValidationError),
}

impl_wrapped_variant!(
    ReferenceUnlockUnpackError,
    ValidationError,
    ReferenceUnlockUnpackError::ValidationError
);

impl fmt::Display for ReferenceUnlockUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// An [`UnlockBlock`](crate::unlock::UnlockBlock) that refers to another [`UnlockBlock`](crate::unlock::UnlockBlock).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ReferenceUnlock(u16);

impl ReferenceUnlock {
    /// The [`UnlockBlock`](crate::unlock::UnlockBlock) kind of a [`ReferenceUnlock`].
    pub const KIND: u8 = 1;

    /// Creates a new [`ReferenceUnlock`].
    pub fn new(index: u16) -> Result<Self, ValidationError> {
        if !UNLOCK_BLOCK_INDEX_RANGE.contains(&index) {
            return Err(ValidationError::InvalidReferenceIndex(index));
        }

        Ok(Self(index))
    }

    /// Returns the index of a [`ReferenceUnlock`].
    pub fn index(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for ReferenceUnlock {
    type Error = ValidationError;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        Self::new(index)
    }
}

impl core::ops::Deref for ReferenceUnlock {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Packable for ReferenceUnlock {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.0.pack(packer).infallible()?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let index = u16::unpack(unpacker).infallible()?;

        ReferenceUnlock::new(index).map_err(|e| UnpackError::Packable(e.into()))
    }
}

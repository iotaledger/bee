// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, unlock::UNLOCK_BLOCK_INDEX_MAX, MessageUnpackError};

use bee_packable::{BoundedU16, InvalidBoundedU16, Packable};

use core::{
    convert::{TryFrom, TryInto},
    fmt,
};

/// Error encountered unpacking a [`ReferenceUnlock`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum ReferenceUnlockUnpackError {
    Validation(ValidationError),
}

impl_wrapped_variant!(
    ReferenceUnlockUnpackError,
    ReferenceUnlockUnpackError::Validation,
    ValidationError
);

impl fmt::Display for ReferenceUnlockUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(e) => write!(f, "{}", e),
        }
    }
}

// TODO would be better as a From but conflicts with OutputId impl
fn invalid_u16_to_validation_error(err: InvalidBoundedU16<0, UNLOCK_BLOCK_INDEX_MAX>) -> ValidationError {
    ValidationError::InvalidReferenceIndex(err.0)
}

/// An [`UnlockBlock`](crate::unlock::UnlockBlock) that refers to another [`UnlockBlock`](crate::unlock::UnlockBlock).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError, with = invalid_u16_to_validation_error)]
pub struct ReferenceUnlock(BoundedU16<0, UNLOCK_BLOCK_INDEX_MAX>);

impl ReferenceUnlock {
    /// The [`UnlockBlock`](crate::unlock::UnlockBlock) kind of a [`ReferenceUnlock`].
    pub const KIND: u8 = 1;

    /// Creates a new [`ReferenceUnlock`].
    pub fn new(index: u16) -> Result<Self, ValidationError> {
        Ok(Self(index.try_into().map_err(invalid_u16_to_validation_error)?))
    }

    /// Returns the index of a [`ReferenceUnlock`].
    pub fn index(&self) -> u16 {
        self.0.into()
    }
}

impl TryFrom<u16> for ReferenceUnlock {
    type Error = ValidationError;

    fn try_from(index: u16) -> Result<Self, Self::Error> {
        Self::new(index)
    }
}

// We cannot provide a `From` implementation because `u16` is an external type.
#[allow(clippy::from_over_into)]
impl Into<u16> for ReferenceUnlock {
    fn into(self) -> u16 {
        self.0.into()
    }
}

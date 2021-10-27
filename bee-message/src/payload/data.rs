// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing a generic data payload.

use crate::{
    payload::{MessagePayload, PAYLOAD_LENGTH_MAX},
    MessageUnpackError, ValidationError,
};

use bee_packable::{
    error::UnpackPrefixError, packable::VecPrefixLengthError, BoundedU32, InvalidBoundedU32, Packable, VecPrefix,
};

use alloc::vec::Vec;
use core::convert::Infallible;

pub(crate) const PREFIXED_DATA_LENGTH_MAX: u32 = PAYLOAD_LENGTH_MAX - core::mem::size_of::<u8>() as u32;

fn unpack_prefix_to_validation_error(
    error: UnpackPrefixError<Infallible, InvalidBoundedU32<0, PREFIXED_DATA_LENGTH_MAX>>,
) -> ValidationError {
    match error {
        UnpackPrefixError::InvalidPrefixLength(len) => {
            ValidationError::InvalidDataPayloadLength(VecPrefixLengthError::Invalid(len))
        }
        UnpackPrefixError::Packable(e) => match e {},
    }
}

/// Generic data payload, containing a collection of bytes.
///
/// A [`DataPayload`] must:
/// * Not exceed [`PAYLOAD_LENGTH_MAX`] in bytes.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError, with = unpack_prefix_to_validation_error)]
pub struct DataPayload {
    /// The raw data in bytes.
    data: VecPrefix<u8, BoundedU32<0, PREFIXED_DATA_LENGTH_MAX>>,
}

impl MessagePayload for DataPayload {
    const KIND: u32 = 1;
    const VERSION: u8 = 0;
}

impl DataPayload {
    /// Creates a new [`DataPayload`].
    pub fn new(data: Vec<u8>) -> Result<Self, ValidationError> {
        Ok(Self {
            data: data.try_into().map_err(ValidationError::InvalidDataPayloadLength)?,
        })
    }

    /// Returns the data bytes of a [`DataPayload`].
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
}

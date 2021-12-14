// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing a generic data payload.

use crate::{
    payload::{MessagePayload, PAYLOAD_LENGTH_MAX},
    MessageUnpackError, ValidationError,
};

use bee_packable::{
    bounded::BoundedU32,
    prefix::{UnpackPrefixError, VecPrefix},
    Packable,
};

use alloc::vec::Vec;
use core::convert::Infallible;

pub(crate) type DataPayloadLength = BoundedU32<0, { PAYLOAD_LENGTH_MAX - core::mem::size_of::<u8>() as u32 }>;

fn unpack_prefix_to_validation_error(
    err: UnpackPrefixError<Infallible, <DataPayloadLength as TryFrom<u32>>::Error>,
) -> ValidationError {
    ValidationError::InvalidDataPayloadLength(err.into_prefix().into())
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
    data: VecPrefix<u8, DataPayloadLength>,
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

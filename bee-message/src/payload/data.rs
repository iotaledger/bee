// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing a generic data payload.

use crate::{
    payload::{MessagePayload, PAYLOAD_LENGTH_MAX},
    MessagePackError, MessageUnpackError, ValidationError,
};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    BoundedU32, InvalidBoundedU32, Packable, VecPrefix,
};

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryInto},
    fmt,
};

const PREFIXED_DATA_LENGTH_MAX: u32 = PAYLOAD_LENGTH_MAX - core::mem::size_of::<u8>() as u32;

/// Error encountered packing a data payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum DataPackError {
    InvalidPrefix,
}

impl From<PackPrefixError<Infallible>> for DataPackError {
    fn from(_: PackPrefixError<Infallible>) -> Self {
        Self::InvalidPrefix
    }
}

impl fmt::Display for DataPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for data"),
        }
    }
}

/// Error encountered unpacking a data payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum DataUnpackError {
    InvalidPrefix,
    InvalidPrefixLength(usize),
}

impl From<UnpackPrefixError<Infallible>> for DataUnpackError {
    fn from(error: UnpackPrefixError<Infallible>) -> Self {
        match error {
            UnpackPrefixError::InvalidPrefixLength(len) => Self::InvalidPrefixLength(len),
            UnpackPrefixError::Packable(e) => match e {},
        }
    }
}

impl fmt::Display for DataUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for data"),
            Self::InvalidPrefixLength(len) => write!(f, "unpacked prefix larger than maximum specified: {}", len),
        }
    }
}

/// Generic data payload, containing a collection of bytes.
///
/// A [`DataPayload`] must:
/// * Not exceed [`PAYLOAD_LENGTH_MAX`] in bytes.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(pack_error = MessagePackError, with = DataPackError::from)]
#[packable(unpack_error = MessageUnpackError, with = DataUnpackError::from)]
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
            data: data
                .try_into()
                // TODO replace ?
                .map_err(|err: InvalidBoundedU32<0, PREFIXED_DATA_LENGTH_MAX>| {
                    ValidationError::InvalidPayloadLength(err.0 as usize)
                })?,
        })
    }

    /// Returns the data bytes of a [`DataPayload`].
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
}

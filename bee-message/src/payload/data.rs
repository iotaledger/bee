// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing a generic data payload.

use crate::{
    payload::{MessagePayload, PAYLOAD_LENGTH_MAX},
    MessagePackError, MessageUnpackError, ValidationError,
};

use bee_packable::{
    coerce::*,
    error::{PackPrefixError, UnpackPrefixError},
    BoundedU32, InvalidBoundedU32, PackError, Packable, Packer, UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryInto},
    fmt,
};

const PREFIXED_DATA_LENGTH_MAX: u32 = (PAYLOAD_LENGTH_MAX - core::mem::size_of::<u8>()) as u32;

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
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
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

impl Packable for DataPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.data.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.data.pack(packer).coerce::<DataPackError>().coerce()?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let data = VecPrefix::<u8, BoundedU32<0, PREFIXED_DATA_LENGTH_MAX>>::unpack(unpacker)
            .coerce::<DataUnpackError>()
            .coerce()?;

        let payload = Self { data };

        Ok(payload)
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing a generic data payload.

use super::PAYLOAD_LENGTH_MAX;
use crate::{MessagePackError, MessageUnpackError, ValidationError};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryInto},
    fmt,
};

const PREFIXED_DATA_LENGTH_MAX: usize = PAYLOAD_LENGTH_MAX - core::mem::size_of::<u8>();

/// Error encountered packing a data payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum DataPackError {
    InvalidPrefix,
}

impl From<PackPrefixError<Infallible, u32>> for DataPackError {
    fn from(_: PackPrefixError<Infallible, u32>) -> Self {
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

impl From<UnpackPrefixError<Infallible, u32>> for DataUnpackError {
    fn from(error: UnpackPrefixError<Infallible, u32>) -> Self {
        match error {
            UnpackPrefixError::InvalidPrefixLength(len) => Self::InvalidPrefixLength(len),
            UnpackPrefixError::Packable(e) => match e {},
            UnpackPrefixError::Prefix(_) => Self::InvalidPrefix,
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
/// A `DataPayload` must:
/// * Not exceed `MAXIMUM_PAYLOAD_LEN` in bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct DataPayload {
    /// The version of the `DataPayload`.
    version: u8,
    /// The raw data in bytes.
    data: Vec<u8>,
}

impl DataPayload {
    /// The payload kind of a `DataPayload`.
    pub const KIND: u32 = 1;

    /// Creates a new `DataPayload`.
    pub fn new(version: u8, data: Vec<u8>) -> Result<Self, ValidationError> {
        validate_data_len(data.len())?;

        Ok(Self { version, data })
    }

    /// Returns the version of a `DataPayload`.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the "data" bytes of a `DataPayload`.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl Packable for DataPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        // Unwrap is safe, since the data length has already been validated.
        let prefixed_data: VecPrefix<u8, u32, PREFIXED_DATA_LENGTH_MAX> = self.data.clone().try_into().unwrap();

        self.version.packed_len() + prefixed_data.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;

        // Unwrap is safe, since the data length has already been validated.
        let prefixed_data: VecPrefix<u8, u32, PREFIXED_DATA_LENGTH_MAX> = self.data.clone().try_into().unwrap();
        prefixed_data
            .pack(packer)
            .map_err(PackError::coerce::<DataPackError>)
            .map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        let data: Vec<u8> = VecPrefix::<u8, u32, PREFIXED_DATA_LENGTH_MAX>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DataUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        validate_data_len(data.len()).map_err(|e| UnpackError::Packable(e.into()))?;

        let payload = Self { version, data };

        Ok(payload)
    }
}

fn validate_data_len(len: usize) -> Result<(), ValidationError> {
    if len > PAYLOAD_LENGTH_MAX {
        Err(ValidationError::InvalidPayloadLength(len))
    } else {
        Ok(())
    }
}

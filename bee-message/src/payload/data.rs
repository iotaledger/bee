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
/// A [`DataPayload`] must:
/// * Not exceed [`PAYLOAD_LENGTH_MAX`] in bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct DataPayload {
    /// The raw data in bytes.
    data: Vec<u8>,
}

impl MessagePayload for DataPayload {
    const KIND: u32 = 1;
    const VERSION: u8 = 0;
}

impl DataPayload {
    /// Creates a new [`DataPayload`].
    pub fn new(data: Vec<u8>) -> Result<Self, ValidationError> {
        validate_data_len(data.len())?;

        Ok(Self { data })
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
        // Unwrap is safe, since the data length has already been validated.
        let prefixed_data: VecPrefix<u8, u32, PREFIXED_DATA_LENGTH_MAX> = self.data.clone().try_into().unwrap();

        Self::VERSION.packed_len() + prefixed_data.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        Self::VERSION.pack(packer).infallible()?;

        // Unwrap is safe, since the data length has already been validated.
        let prefixed_data: VecPrefix<u8, u32, PREFIXED_DATA_LENGTH_MAX> = self.data.clone().try_into().unwrap();
        prefixed_data.pack(packer).coerce::<DataPackError>().coerce()?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).infallible()?;
        validate_payload_version(version).map_err(|e| UnpackError::Packable(e.into()))?;

        let data: Vec<u8> = VecPrefix::<u8, u32, PREFIXED_DATA_LENGTH_MAX>::unpack(unpacker)
            .coerce::<DataUnpackError>()
            .coerce()?
            .into();
        validate_data_len(data.len()).map_err(|e| UnpackError::Packable(e.into()))?;

        let payload = Self { data };

        Ok(payload)
    }
}

fn validate_payload_version(version: u8) -> Result<(), ValidationError> {
    if version != DataPayload::VERSION {
        Err(ValidationError::InvalidPayloadVersion {
            version,
            payload_kind: DataPayload::KIND,
        })
    } else {
        Ok(())
    }
}

fn validate_data_len(len: usize) -> Result<(), ValidationError> {
    if len > PREFIXED_DATA_LENGTH_MAX {
        Err(ValidationError::InvalidPayloadLength(len))
    } else {
        Ok(())
    }
}

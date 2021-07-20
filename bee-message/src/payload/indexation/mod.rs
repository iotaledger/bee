// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

mod padded;

use crate::{MessagePackError, MessageUnpackError, ValidationError, MESSAGE_LENGTH_RANGE};

pub use padded::{PaddedIndex, INDEXATION_PADDED_INDEX_LENGTH};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryInto},
    fmt,
    ops::RangeInclusive,
};

/// Valid lengths for an indexation payload index.
pub const INDEXATION_INDEX_LENGTH_RANGE: RangeInclusive<usize> = 1..=INDEXATION_PADDED_INDEX_LENGTH;

const PREFIXED_INDEX_LENGTH_MAX: usize = *INDEXATION_INDEX_LENGTH_RANGE.end();
const PREFIXED_DATA_LENGTH_MAX: usize = *MESSAGE_LENGTH_RANGE.end();

/// Error encountered packing an indexation payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum IndexationPackError {
    InvalidPrefix,
}

impl From<PackPrefixError<Infallible, u32>> for IndexationPackError {
    fn from(error: PackPrefixError<Infallible, u32>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {},
            PackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

impl fmt::Display for IndexationPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for index/data"),
        }
    }
}

/// Error encountered unpacking an indexation payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum IndexationUnpackError {
    InvalidPrefix,
    InvalidPrefixLength(usize),
    ValidationError(ValidationError),
}

impl_wrapped_variant!(
    IndexationUnpackError,
    ValidationError,
    IndexationUnpackError::ValidationError
);

impl From<UnpackPrefixError<Infallible, u32>> for IndexationUnpackError {
    fn from(error: UnpackPrefixError<Infallible, u32>) -> Self {
        match error {
            UnpackPrefixError::InvalidPrefixLength(len) => Self::InvalidPrefixLength(len),
            UnpackPrefixError::Packable(e) => match e {},
            UnpackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

impl fmt::Display for IndexationUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for index/data"),
            Self::InvalidPrefixLength(len) => write!(f, "unpacked prefix larger than maximum specified: {}", len),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A payload which holds an index and associated data.
///
/// An `IndexationPayload` must:
/// * Contain an index of within `INDEXATION_INDEX_LENGTH_RANGE` bytes.
/// * Contain data that does not exceed maximum message length in bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "enable-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexationPayload {
    /// The version of the `IndexationPayload`.
    version: u8,
    /// The index key of the message.
    index: Vec<u8>,
    /// The data attached to this index.
    data: Vec<u8>,
}

impl IndexationPayload {
    /// The payload kind of an `IndexationPayload`.
    pub const KIND: u32 = 8;

    /// Creates a new `IndexationPayload`.
    pub fn new(version: u8, index: Vec<u8>, data: Vec<u8>) -> Result<Self, ValidationError> {
        validate_index(&index)?;
        validate_data(&data)?;

        Ok(Self { version, index, data })
    }

    /// Returns the index of an `IndexationPayload`.
    pub fn index(&self) -> &[u8] {
        &self.index
    }

    /// Returns the padded index of an `IndexationPayload`.
    pub fn padded_index(&self) -> PaddedIndex {
        let mut padded_index = [0u8; INDEXATION_PADDED_INDEX_LENGTH];
        padded_index[..self.index.len()].copy_from_slice(&self.index);
        PaddedIndex::from(padded_index)
    }

    /// Returns the data of an `IndexationPayload`.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

fn validate_index(index: &[u8]) -> Result<(), ValidationError> {
    if !INDEXATION_INDEX_LENGTH_RANGE.contains(&index.len()) {
        Err(ValidationError::InvalidIndexationIndexLength(index.len()))
    } else {
        Ok(())
    }
}

fn validate_data(data: &[u8]) -> Result<(), ValidationError> {
    if data.len() > *MESSAGE_LENGTH_RANGE.end() {
        Err(ValidationError::InvalidIndexationDataLength(data.len()))
    } else {
        Ok(())
    }
}

impl Packable for IndexationPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        // Unwrap is safe, since index/data lengths have already been validated.
        let prefixed_index: VecPrefix<u8, u32, PREFIXED_INDEX_LENGTH_MAX> = self.index.clone().try_into().unwrap();
        let prefixed_data: VecPrefix<u8, u32, PREFIXED_DATA_LENGTH_MAX> = self.data.clone().try_into().unwrap();

        self.version.packed_len() + prefixed_index.packed_len() + prefixed_data.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;

        // Unwrap is safe, since index/data lengths have already been validated.
        let prefixed_index: VecPrefix<u8, u32, PREFIXED_INDEX_LENGTH_MAX> = self.index.clone().try_into().unwrap();
        prefixed_index
            .pack(packer)
            .map_err(PackError::coerce::<IndexationPackError>)
            .map_err(PackError::coerce)?;

        let prefixed_data: VecPrefix<u8, u32, PREFIXED_DATA_LENGTH_MAX> = self.data.clone().try_into().unwrap();
        prefixed_data
            .pack(packer)
            .map_err(PackError::coerce::<IndexationPackError>)
            .map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;

        let index: Vec<u8> = VecPrefix::<u8, u32, PREFIXED_INDEX_LENGTH_MAX>::unpack(unpacker)
            .map_err(UnpackError::coerce::<IndexationUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        validate_index(&index).map_err(|e| UnpackError::Packable(e.into()))?;

        let data: Vec<u8> = VecPrefix::<u8, u32, PREFIXED_DATA_LENGTH_MAX>::unpack(unpacker)
            .map_err(UnpackError::coerce::<IndexationUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        validate_data(&data).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { version, index, data })
    }
}

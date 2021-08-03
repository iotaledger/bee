// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

mod padded;

use crate::{payload::MessagePayload, MessagePackError, MessageUnpackError, ValidationError, MESSAGE_LENGTH_RANGE};

pub use padded::PaddedIndex;

use bee_packable::{
    coerce::*,
    error::{PackPrefixError, UnpackPrefixError},
    BoundedU32, InvalidBoundedU32, PackError, Packable, Packer, UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryInto},
    fmt,
    ops::RangeInclusive,
};

/// Valid lengths for an indexation payload index.
pub const INDEXATION_INDEX_LENGTH_RANGE: RangeInclusive<u32> = 1..=PaddedIndex::LENGTH as u32;

const PREFIXED_INDEX_LENGTH_MIN: u32 = *INDEXATION_INDEX_LENGTH_RANGE.start() as u32;
const PREFIXED_DATA_LENGTH_MIN: u32 = *MESSAGE_LENGTH_RANGE.start() as u32;
const PREFIXED_INDEX_LENGTH_MAX: u32 = *INDEXATION_INDEX_LENGTH_RANGE.end() as u32;
const PREFIXED_DATA_LENGTH_MAX: u32 = *MESSAGE_LENGTH_RANGE.end() as u32;

/// Error encountered packing an indexation payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum IndexationPackError {
    InvalidPrefix,
}

impl From<PackPrefixError<Infallible>> for IndexationPackError {
    fn from(error: PackPrefixError<Infallible>) -> Self {
        match error.0 {}
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

impl From<UnpackPrefixError<Infallible>> for IndexationUnpackError {
    fn from(error: UnpackPrefixError<Infallible>) -> Self {
        match error {
            UnpackPrefixError::InvalidPrefixLength(len) => Self::InvalidPrefixLength(len),
            UnpackPrefixError::Packable(e) => match e {},
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
/// An [`IndexationPayload`] must:
/// * Contain an index of within [`INDEXATION_INDEX_LENGTH_RANGE`] bytes.
/// * Contain data that does not exceed maximum message length in bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexationPayload {
    /// The index key of the message.
    index: VecPrefix<u8, BoundedU32<PREFIXED_INDEX_LENGTH_MIN, PREFIXED_INDEX_LENGTH_MAX>>,
    /// The data attached to this index.
    data: VecPrefix<u8, BoundedU32<PREFIXED_DATA_LENGTH_MIN, PREFIXED_DATA_LENGTH_MAX>>,
}

impl MessagePayload for IndexationPayload {
    const KIND: u32 = 8;
    const VERSION: u8 = 0;
}

impl IndexationPayload {
    /// Creates a new [`IndexationPayload`].
    pub fn new(index: Vec<u8>, data: Vec<u8>) -> Result<Self, ValidationError> {
        Ok(Self {
            index: index.try_into().map_err(
                |err: InvalidBoundedU32<PREFIXED_INDEX_LENGTH_MIN, PREFIXED_INDEX_LENGTH_MAX>| {
                    ValidationError::InvalidIndexationIndexLength(err.0 as usize)
                },
            )?,
            data: data.try_into().map_err(
                |err: InvalidBoundedU32<PREFIXED_DATA_LENGTH_MIN, PREFIXED_DATA_LENGTH_MAX>| {
                    ValidationError::InvalidIndexationDataLength(err.0 as usize)
                },
            )?,
        })
    }

    /// Returns the index of an [`IndexationPayload`].
    pub fn index(&self) -> &[u8] {
        &self.index
    }

    /// Returns the padded index of an [`IndexationPayload`].
    pub fn padded_index(&self) -> PaddedIndex {
        let mut padded_index = [0u8; PaddedIndex::LENGTH];
        padded_index[..self.index.len()].copy_from_slice(&self.index);
        PaddedIndex::from(padded_index)
    }

    /// Returns the data of an [`IndexationPayload`].
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Packable for IndexationPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.index.packed_len() + self.data.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.index.pack(packer).coerce::<IndexationPackError>().coerce()?;

        self.data.pack(packer).coerce::<IndexationPackError>().coerce()?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let index = VecPrefix::<u8, BoundedU32<PREFIXED_INDEX_LENGTH_MIN, PREFIXED_INDEX_LENGTH_MAX>>::unpack(unpacker)
            .coerce::<IndexationUnpackError>()
            .coerce()?;

        let data = VecPrefix::<u8, BoundedU32<PREFIXED_DATA_LENGTH_MIN, PREFIXED_DATA_LENGTH_MAX>>::unpack(unpacker)
            .coerce::<IndexationUnpackError>()
            .coerce()?;

        Ok(Self { index, data })
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

mod padded;

use crate::{payload::MessagePayload, MessageUnpackError, ValidationError, MESSAGE_LENGTH_RANGE};

pub use padded::PaddedIndex;

use bee_packable::{
    bounded::{BoundedU32, InvalidBoundedU32},
    prefix::{TryIntoPrefixError, UnpackPrefixError, VecPrefix},
    Packable,
};

use alloc::vec::Vec;
use core::{convert::Infallible, ops::RangeInclusive};

/// Valid lengths for an indexation payload index.
pub const INDEXATION_INDEX_LENGTH_RANGE: RangeInclusive<u32> = 1..=PaddedIndex::LENGTH as u32;

pub(crate) const PREFIXED_INDEXATION_INDEX_LENGTH_MIN: u32 = *INDEXATION_INDEX_LENGTH_RANGE.start() as u32;
pub(crate) const PREFIXED_INDEXATION_INDEX_LENGTH_MAX: u32 = *INDEXATION_INDEX_LENGTH_RANGE.end() as u32;
pub(crate) const PREFIXED_INDEXATION_DATA_LENGTH_MAX: u32 = *MESSAGE_LENGTH_RANGE.end() as u32;

fn unpack_prefix_to_invalid_index_length(
    err: UnpackPrefixError<
        Infallible,
        InvalidBoundedU32<PREFIXED_INDEXATION_INDEX_LENGTH_MIN, PREFIXED_INDEXATION_INDEX_LENGTH_MAX>,
    >,
) -> ValidationError {
    ValidationError::InvalidIndexationIndexLength(TryIntoPrefixError::Invalid(err.into_prefix()))
}

fn unpack_prefix_to_invalid_data_length(
    err: UnpackPrefixError<Infallible, InvalidBoundedU32<0, PREFIXED_INDEXATION_DATA_LENGTH_MAX>>,
) -> ValidationError {
    ValidationError::InvalidIndexationDataLength(TryIntoPrefixError::Invalid(err.into_prefix()))
}

/// A payload which holds an index and associated data.
///
/// An [`IndexationPayload`] must:
/// * Contain an index of within [`INDEXATION_INDEX_LENGTH_RANGE`] bytes.
/// * Contain data that does not exceed maximum message length in bytes.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct IndexationPayload {
    /// The index key of the message.
    #[packable(unpack_error_with = unpack_prefix_to_invalid_index_length)]
    index: VecPrefix<u8, BoundedU32<PREFIXED_INDEXATION_INDEX_LENGTH_MIN, PREFIXED_INDEXATION_INDEX_LENGTH_MAX>>,
    /// The data attached to this index.
    #[packable(unpack_error_with = unpack_prefix_to_invalid_data_length)]
    data: VecPrefix<u8, BoundedU32<0, PREFIXED_INDEXATION_DATA_LENGTH_MAX>>,
}

impl MessagePayload for IndexationPayload {
    const KIND: u32 = 8;
    const VERSION: u8 = 0;
}

impl IndexationPayload {
    /// Creates a new [`IndexationPayload`].
    pub fn new(index: Vec<u8>, data: Vec<u8>) -> Result<Self, ValidationError> {
        Ok(Self {
            index: index
                .try_into()
                .map_err(ValidationError::InvalidIndexationIndexLength)?,
            data: data.try_into().map_err(ValidationError::InvalidIndexationDataLength)?,
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

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

mod padded;

use crate::{Error, MESSAGE_LENGTH_MAX};

pub use padded::{PaddedIndex, INDEXATION_PADDED_INDEX_LENGTH};

use bee_packable::{
    bounded::{BoundedU16, BoundedU32, InvalidBoundedU16, InvalidBoundedU32},
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::{TryIntoPrefixError, UnpackPrefixError, VecPrefix},
    unpacker::Unpacker,
    Packable,
};

use alloc::boxed::Box;
use core::ops::Range;
use std::{
    convert::{Infallible, TryInto},
    ops::RangeInclusive,
};

/// Valid lengths for an indexation payload index.
pub const INDEXATION_INDEX_LENGTH_RANGE: RangeInclusive<u16> =
    INDEXATION_INDEX_LENGTH_MIN..=INDEXATION_INDEX_LENGTH_MAX;
pub const INDEXATION_INDEX_LENGTH_MIN: u16 = 1;
pub const INDEXATION_INDEX_LENGTH_MAX: u16 = INDEXATION_PADDED_INDEX_LENGTH as u16;
pub const INDEXATION_DATA_LENGTH_MAX: u32 = MESSAGE_LENGTH_MAX as u32;

fn unpack_prefix_to_invalid_index_length(
    err: UnpackPrefixError<Infallible, InvalidBoundedU16<INDEXATION_INDEX_LENGTH_MIN, INDEXATION_INDEX_LENGTH_MAX>>,
) -> Error {
    Error::InvalidIndexationIndexLength(TryIntoPrefixError::Invalid(err.into_prefix()))
}

fn unpack_prefix_to_invalid_data_length(
    err: UnpackPrefixError<Infallible, InvalidBoundedU32<0, INDEXATION_DATA_LENGTH_MAX>>,
) -> Error {
    Error::InvalidIndexationDataLength(TryIntoPrefixError::Invalid(err.into_prefix()))
}

/// A payload which holds an index and associated data.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct IndexationPayload {
    #[packable(unpack_error_with = unpack_prefix_to_invalid_index_length)]
    index: VecPrefix<u8, BoundedU16<INDEXATION_INDEX_LENGTH_MIN, INDEXATION_INDEX_LENGTH_MAX>>,
    #[packable(unpack_error_with = unpack_prefix_to_invalid_data_length)]
    data: VecPrefix<u8, BoundedU32<0, INDEXATION_DATA_LENGTH_MAX>>,
}

impl IndexationPayload {
    /// The payload kind of an `IndexationPayload`.
    pub const KIND: u32 = 2;

    /// Creates a new `IndexationPayload`.
    pub fn new(index: &[u8], data: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            index: index.to_vec().try_into().map_err(Error::InvalidIndexationIndexLength)?,
            data: data.to_vec().try_into().map_err(Error::InvalidIndexationDataLength)?,
        })
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

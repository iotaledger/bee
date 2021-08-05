// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

mod padded;

use crate::{payload::MessagePayload, MessageUnpackError, ValidationError, MESSAGE_LENGTH_RANGE};

pub use padded::PaddedIndex;

use bee_packable::{
    coerce::PackCoerceInfallible, error::UnpackPrefixError, BoundedU32, InvalidBoundedU32, PackError, Packable, Packer,
    UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryInto},
    ops::RangeInclusive,
};

/// Valid lengths for an indexation payload index.
pub const INDEXATION_INDEX_LENGTH_RANGE: RangeInclusive<u32> = 1..=PaddedIndex::LENGTH as u32;

const PREFIXED_INDEX_LENGTH_MIN: u32 = *INDEXATION_INDEX_LENGTH_RANGE.start() as u32;
const PREFIXED_INDEX_LENGTH_MAX: u32 = *INDEXATION_INDEX_LENGTH_RANGE.end() as u32;
const PREFIXED_DATA_LENGTH_MAX: u32 = *MESSAGE_LENGTH_RANGE.end() as u32;

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
    data: VecPrefix<u8, BoundedU32<0, PREFIXED_DATA_LENGTH_MAX>>,
}

impl MessagePayload for IndexationPayload {
    const KIND: u32 = 8;
    const VERSION: u8 = 0;
}

impl Packable for IndexationPayload {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.index.packed_len() + self.data.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.index.pack(packer).infallible()?;
        self.data.pack(packer).infallible()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let index = VecPrefix::<u8, BoundedU32<PREFIXED_INDEX_LENGTH_MIN, PREFIXED_INDEX_LENGTH_MAX>>::unpack(unpacker)
            .map_err(|unpack_err| {
                unpack_err.map(|err| match err {
                    UnpackPrefixError::InvalidPrefixLength(len) => {
                        ValidationError::InvalidIndexationIndexLength(len).into()
                    }
                    UnpackPrefixError::Packable(e) => match e {},
                })
            })?;

        let data =
            VecPrefix::<u8, BoundedU32<0, PREFIXED_DATA_LENGTH_MAX>>::unpack(unpacker).map_err(|unpack_err| {
                unpack_err.map(|err| match err {
                    UnpackPrefixError::InvalidPrefixLength(len) => {
                        ValidationError::InvalidIndexationDataLength(len).into()
                    }
                    UnpackPrefixError::Packable(e) => match e {},
                })
            })?;

        Ok(Self { index, data })
    }
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
            data: data
                .try_into()
                .map_err(|err: InvalidBoundedU32<0, PREFIXED_DATA_LENGTH_MAX>| {
                    ValidationError::InvalidIndexationDataLength(err.0 as usize)
                })?,
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

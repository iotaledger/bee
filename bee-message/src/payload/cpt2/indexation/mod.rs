// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

mod padded;

pub use padded::PaddedIndex;

use crate::{Error, Message};

use packable::{
    bounded::{BoundedU32, BoundedU8},
    prefix::BoxedSlicePrefix,
    Packable,
};

use core::ops::RangeInclusive;

#[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
pub(crate) type IndexLength =
    BoundedU8<{ *IndexationPayload::LENGTH_RANGE.start() }, { *IndexationPayload::LENGTH_RANGE.end() }>;
#[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
pub(crate) type IndexationDataLength = BoundedU32<0, { Message::LENGTH_MAX as u32 }>;

/// A payload which holds an index and associated data.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
pub struct IndexationPayload {
    #[packable(unpack_error_with = |err| Error::InvalidIndexLength(err.into_prefix().into()))]
    index: BoxedSlicePrefix<u8, IndexLength>,
    #[packable(unpack_error_with = |err| Error::InvalidIndexationDataLength(err.into_prefix().into()))]
    data: BoxedSlicePrefix<u8, IndexationDataLength>,
}

impl IndexationPayload {
    /// The payload kind of an [`IndexationPayload`].
    pub const KIND: u32 = 2;
    /// Valid lengths for an index.
    pub const LENGTH_RANGE: RangeInclusive<u8> = 1..=64;

    /// Creates a new [`IndexationPayload`].
    pub fn new(index: Vec<u8>, data: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            index: index.into_boxed_slice().try_into().map_err(Error::InvalidIndexLength)?,
            data: data
                .into_boxed_slice()
                .try_into()
                .map_err(Error::InvalidIndexationDataLength)?,
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

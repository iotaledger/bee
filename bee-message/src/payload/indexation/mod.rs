// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

use crate::{Error, Message};

use packable::{
    bounded::{BoundedU16, BoundedU32},
    prefix::BoxedSlicePrefix,
};

use core::ops::RangeInclusive;

pub(crate) type IndexationIndexLength =
    BoundedU16<{ *IndexationPayload::LENGTH_RANGE.start() }, { *IndexationPayload::LENGTH_RANGE.end() }>;
pub(crate) type IndexationDataLength = BoundedU32<0, { Message::LENGTH_MAX as u32 }>;

/// A payload which holds an index and associated data.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct IndexationPayload {
    #[packable(unpack_error_with = |err| Error::InvalidIndexationIndexLength(err.into_prefix().into()))]
    index: BoxedSlicePrefix<u8, IndexationIndexLength>,
    #[packable(unpack_error_with = |err| Error::InvalidIndexationDataLength(err.into_prefix().into()))]
    data: BoxedSlicePrefix<u8, IndexationDataLength>,
}

impl IndexationPayload {
    /// The payload kind of an `IndexationPayload`.
    pub const KIND: u32 = 2;
    /// Valid lengths for an indexation payload index.
    pub const LENGTH_RANGE: RangeInclusive<u16> = 1..=64;

    /// Creates a new `IndexationPayload`.
    pub fn new(index: Vec<u8>, data: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            index: index
                .into_boxed_slice()
                .try_into()
                .map_err(Error::InvalidIndexationIndexLength)?,
            data: data
                .into_boxed_slice()
                .try_into()
                .map_err(Error::InvalidIndexationDataLength)?,
        })
    }

    /// Returns the index of an `IndexationPayload`.
    pub fn index(&self) -> &[u8] {
        &self.index
    }

    /// Returns the data of an `IndexationPayload`.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

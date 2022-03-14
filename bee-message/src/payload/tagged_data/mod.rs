// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the tagged data payload.

use crate::{Error, Message};

use packable::{
    bounded::{BoundedU32, BoundedU8},
    prefix::BoxedSlicePrefix,
    Packable,
};

use alloc::vec::Vec;
use core::ops::RangeInclusive;

pub(crate) type TagLength =
    BoundedU8<{ *TaggedDataPayload::TAG_LENGTH_RANGE.start() }, { *TaggedDataPayload::TAG_LENGTH_RANGE.end() }>;
pub(crate) type TaggedDataLength =
    BoundedU32<{ *TaggedDataPayload::DATA_LENGTH_RANGE.start() }, { *TaggedDataPayload::DATA_LENGTH_RANGE.end() }>;

/// A payload which holds a tag and associated data.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct TaggedDataPayload {
    #[packable(unpack_error_with = |err| Error::InvalidTagLength(err.into_prefix_err().into()))]
    tag: BoxedSlicePrefix<u8, TagLength>,
    #[packable(unpack_error_with = |err| Error::InvalidTaggedDataLength(err.into_prefix_err().into()))]
    data: BoxedSlicePrefix<u8, TaggedDataLength>,
}

impl TaggedDataPayload {
    /// The payload kind of a [`TaggedDataPayload`].
    pub const KIND: u32 = 5;
    /// Valid lengths for the tag.
    pub const TAG_LENGTH_RANGE: RangeInclusive<u8> = 0..=64;
    /// Valid lengths for the data.
    pub const DATA_LENGTH_RANGE: RangeInclusive<u32> = 0..=Message::LENGTH_MAX as u32;

    /// Creates a new [`TaggedDataPayload`].
    pub fn new(tag: Vec<u8>, data: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            tag: tag.into_boxed_slice().try_into().map_err(Error::InvalidTagLength)?,
            data: data
                .into_boxed_slice()
                .try_into()
                .map_err(Error::InvalidTaggedDataLength)?,
        })
    }

    /// Returns the tag of a [`TaggedDataPayload`].
    pub fn tag(&self) -> &[u8] {
        &self.tag
    }

    /// Returns the data of a [`TaggedDataPayload`].
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

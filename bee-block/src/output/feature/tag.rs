// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix};

use crate::Error;

pub(crate) type TagFeatureLength =
    BoundedU8<{ *TagFeature::LENGTH_RANGE.start() }, { *TagFeature::LENGTH_RANGE.end() }>;

/// Makes it possible to tag outputs with an index, so they can be retrieved through an indexer API.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| Error::InvalidTagFeatureLength(e.into_prefix_err().into()))]
pub struct TagFeature(
    // Binary tag.
    BoxedSlicePrefix<u8, TagFeatureLength>,
);

impl TryFrom<Vec<u8>> for TagFeature {
    type Error = Error;

    fn try_from(tag: Vec<u8>) -> Result<Self, Error> {
        tag.into_boxed_slice()
            .try_into()
            .map(Self)
            .map_err(Error::InvalidTagFeatureLength)
    }
}

impl TagFeature {
    /// The [`Feature`](crate::output::Feature) kind of an [`TagFeature`].
    pub const KIND: u8 = 3;
    /// Valid lengths for an [`TagFeature`].
    pub const LENGTH_RANGE: RangeInclusive<u8> = 1..=64;

    /// Creates a new [`TagFeature`].
    #[inline(always)]
    pub fn new(tag: Vec<u8>) -> Result<Self, Error> {
        Self::try_from(tag)
    }

    /// Returns the tag.
    #[inline(always)]
    pub fn tag(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Display for TagFeature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", prefix_hex::encode(self.tag()))
    }
}

impl core::fmt::Debug for TagFeature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "TagFeature({})", self)
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct TagFeatureDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub tag: String,
    }
}

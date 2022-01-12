// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix};

use core::ops::RangeInclusive;

pub(crate) type IndexationFeatureBlockLength =
BoundedU8<{ *IndexationFeatureBlock::LENGTH_RANGE.start() }, { *IndexationFeatureBlock::LENGTH_RANGE.end() }>;

/// Defines an indexation tag to which the output will be indexed.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, bee_packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| Error::InvalidIndexationFeatureBlockLength(e.into_prefix().into()))]
pub struct IndexationFeatureBlock(
    // Binary indexation tag.
    BoxedSlicePrefix<u8, IndexationFeatureBlockLength>,
);

impl TryFrom<Vec<u8>> for IndexationFeatureBlock {
    type Error = Error;

    fn try_from(tag: Vec<u8>) -> Result<Self, Error> {
        tag.into_boxed_slice()
            .try_into()
            .map(Self)
            .map_err(Error::InvalidIndexationFeatureBlockLength)
    }
}

impl IndexationFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of an [`IndexationFeatureBlock`].
    pub const KIND: u8 = 8;

    /// Valid lengths for an [`IndexationFeatureBlock`].
    pub const LENGTH_RANGE: RangeInclusive<u8> = 1..=64;

    /// Creates a new [`IndexationFeatureBlock`].
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

impl core::fmt::Display for IndexationFeatureBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl core::fmt::Debug for IndexationFeatureBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "IndexationFeatureBlock({})", self)
    }
}
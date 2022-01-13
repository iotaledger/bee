// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_packable::{bounded::BoundedU32, prefix::BoxedSlicePrefix};

use core::ops::RangeInclusive;

pub(crate) type MetadataFeatureBlockLength =
BoundedU32<{ *MetadataFeatureBlock::LENGTH_RANGE.start() }, { *MetadataFeatureBlock::LENGTH_RANGE.end() }>;

/// Defines metadata, arbitrary binary data, that will be stored in the output.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, bee_packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |err| Error::InvalidMetadataFeatureBlockLength(err.into_prefix().into()))]
pub struct MetadataFeatureBlock(
    // Binary data.
    BoxedSlicePrefix<u8, MetadataFeatureBlockLength>,
);

impl TryFrom<Vec<u8>> for MetadataFeatureBlock {
    type Error = Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Error> {
        data.into_boxed_slice()
            .try_into()
            .map(Self)
            .map_err(Error::InvalidMetadataFeatureBlockLength)
    }
}

impl MetadataFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of [`MetadataFeatureBlock`].
    pub const KIND: u8 = 7;

    /// Valid lengths for a [`MetadataFeatureBlock`].
    pub const LENGTH_RANGE: RangeInclusive<u32> = 1..=1024;

    /// Creates a new [`MetadataFeatureBlock`].
    #[inline(always)]
    pub fn new(data: Vec<u8>) -> Result<Self, Error> {
        Self::try_from(data)
    }

    /// Returns the data.
    #[inline(always)]
    pub fn data(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Display for MetadataFeatureBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(&*self.0))
    }
}

impl core::fmt::Debug for MetadataFeatureBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MetadataFeatureBlock({})", self)
    }
}
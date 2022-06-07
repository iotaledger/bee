// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use packable::{bounded::BoundedU16, prefix::BoxedSlicePrefix};

use crate::Error;

pub(crate) type MetadataFeatureLength =
    BoundedU16<{ *MetadataFeature::LENGTH_RANGE.start() }, { *MetadataFeature::LENGTH_RANGE.end() }>;

/// Defines metadata, arbitrary binary data, that will be stored in the output.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |err| Error::InvalidMetadataFeatureLength(err.into_prefix_err().into()))]
pub struct MetadataFeature(
    // Binary data.
    BoxedSlicePrefix<u8, MetadataFeatureLength>,
);

impl TryFrom<Vec<u8>> for MetadataFeature {
    type Error = Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Error> {
        data.into_boxed_slice()
            .try_into()
            .map(Self)
            .map_err(Error::InvalidMetadataFeatureLength)
    }
}

impl MetadataFeature {
    /// The [`Feature`](crate::output::Feature) kind of [`MetadataFeature`].
    pub const KIND: u8 = 2;
    /// Valid lengths for a [`MetadataFeature`].
    pub const LENGTH_RANGE: RangeInclusive<u16> = 1..=8192;

    /// Creates a new [`MetadataFeature`].
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

impl core::fmt::Display for MetadataFeature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", prefix_hex::encode(self.data()))
    }
}

impl core::fmt::Debug for MetadataFeature {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MetadataFeature({})", self)
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct MetadataFeatureDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub data: String,
    }
}

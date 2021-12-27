// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable as OldPackable, Read, Write};
use bee_packable::{bounded::BoundedU32, prefix::BoxedSlicePrefix};

pub(crate) type MetadataFeatureBlockLength =
    BoundedU32<{ *MetadataFeatureBlock::LENGTH_RANGE.start() }, { *MetadataFeatureBlock::LENGTH_RANGE.end() }>;

use core::ops::RangeInclusive;

/// Defines metadata, arbitrary binary data, that will be stored in the output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, bee_packable::Packable)]
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

impl OldPackable for MetadataFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u32.packed_len() + self.0.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.0.len() as u32).pack(writer)?;
        writer.write_all(&self.0)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let data_length = u32::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK {
            validate_length(data_length)?;
        }

        let mut data = vec![0u8; data_length];
        reader.read_exact(&mut data)?;

        Self::new(data)
    }
}

#[inline]
fn validate_length(data_length: usize) -> Result<(), Error> {
    MetadataFeatureBlockLength::try_from(data_length).map_err(Error::InvalidMetadataFeatureBlockLength)?;

    Ok(())
}

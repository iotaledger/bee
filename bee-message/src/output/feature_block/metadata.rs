// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

/// Defines metadata, arbitrary binary data, that will be stored in the output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MetadataFeatureBlock(
    // Binary data.
    Box<[u8]>,
);

impl TryFrom<&[u8]> for MetadataFeatureBlock {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Error> {
        validate_length(data.len())?;

        Ok(MetadataFeatureBlock(data.into()))
    }
}

impl MetadataFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of [`MetadataFeatureBlock`].
    pub const KIND: u8 = 7;
    /// Maximum possible length in bytes of the data field.
    pub const LENGTH_MAX: usize = 1024;

    /// Creates a new [`MetadataFeatureBlock`].
    #[inline(always)]
    pub fn new(data: &[u8]) -> Result<Self, Error> {
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
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl core::fmt::Debug for MetadataFeatureBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MetadataFeatureBlock({})", self)
    }
}

impl Packable for MetadataFeatureBlock {
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

        Ok(Self(data.into()))
    }
}

#[inline]
fn validate_length(data_length: usize) -> Result<(), Error> {
    if data_length == 0 || data_length > MetadataFeatureBlock::LENGTH_MAX {
        return Err(Error::InvalidMetadataLength(data_length));
    }

    Ok(())
}

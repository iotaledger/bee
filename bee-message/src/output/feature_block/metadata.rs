// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

///
const METADATA_LENGTH_MAX: u32 = 1024;

/// Defines metadata (arbitrary binary data) that will be stored in the output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MetadataFeatureBlock {
    // Binary data.
    data: Box<[u8]>,
}

impl TryFrom<&[u8]> for MetadataFeatureBlock {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Error> {
        if data.len() == 0 || data.len() > METADATA_LENGTH_MAX as usize {
            return Err(Error::InvalidMetadataLength(data.len() as u32));
        }

        Ok(MetadataFeatureBlock { data: data.into() })
    }
}

impl MetadataFeatureBlock {
    /// The [`FeatureBlock`] kind of [`MetadataFeatureBlock`].
    pub const KIND: u8 = 7;

    /// Creates a new [`MetadataFeatureBlock`].
    pub fn new(data: &[u8]) -> Result<Self, Error> {
        data.try_into()
    }

    /// Returns the data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Packable for MetadataFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u32.packed_len() + self.data.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.data.len() as u32).pack(writer)?;
        writer.write_all(&self.data)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let data_len = u32::unpack_inner::<R, CHECK>(reader)?;
        if data_len == 0 || data_len > METADATA_LENGTH_MAX {
            return Err(Error::InvalidMetadataLength(data_len));
        }
        let mut data = vec![0u8; data_len as usize];
        reader.read_exact(&mut data)?;

        Ok(Self { data: data.into() })
    }
}

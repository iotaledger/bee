// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MetadataFeatureBlock(Box<[u8]>);

impl From<&[u8]> for MetadataFeatureBlock {
    fn from(data: &[u8]) -> Self {
        MetadataFeatureBlock(data.into())
    }
}

impl MetadataFeatureBlock {
    /// The feature block kind of a `MetadataFeatureBlock`.
    pub const KIND: u8 = 8;

    /// Creates a new `MetadataFeatureBlock`.
    pub fn new(data: &[u8]) -> Self {
        data.into()
    }

    /// Returns the data.
    pub fn data(&self) -> &[u8] {
        &self.0
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
        let data_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut data = vec![0u8; data_len];
        reader.read_exact(&mut data)?;

        Ok(Self(data.into()))
    }
}

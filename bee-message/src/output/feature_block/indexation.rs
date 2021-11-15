// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexationFeatureBlock(Box<[u8]>);

impl From<&[u8]> for IndexationFeatureBlock {
    fn from(tag: &[u8]) -> Self {
        IndexationFeatureBlock(tag.into())
    }
}

impl IndexationFeatureBlock {
    /// The feature block kind of an `IndexationFeatureBlock`.
    pub const KIND: u8 = 7;

    /// Creates a new `IndexationFeatureBlock`.
    pub fn new(tag: &[u8]) -> Self {
        tag.into()
    }

    /// Returns the tag.
    pub fn tag(&self) -> &[u8] {
        &self.0
    }
}

impl Packable for IndexationFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + self.0.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.0.len() as u8).pack(writer)?;
        writer.write_all(&self.0)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let tag_len = u8::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut tag = vec![0u8; tag_len];
        reader.read_exact(&mut tag)?;

        Ok(Self(tag.into()))
    }
}

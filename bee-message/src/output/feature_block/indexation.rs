// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

/// Defines an indexation tag to which the output will be indexed.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexationFeatureBlock {
    // Binary indexation tag.
    tag: Box<[u8]>,
}

impl TryFrom<&[u8]> for IndexationFeatureBlock {
    type Error = Error;

    fn try_from(tag: &[u8]) -> Result<Self, Error> {
        validate_length(tag.len())?;

        Ok(IndexationFeatureBlock { tag: tag.into() })
    }
}

impl IndexationFeatureBlock {
    /// The [`FeatureBlock`] kind of an [`IndexationFeatureBlock`].
    pub const KIND: u8 = 8;
    ///
    pub const LENGTH_MAX: usize = 64;

    /// Creates a new [`IndexationFeatureBlock`].
    pub fn new(tag: &[u8]) -> Result<Self, Error> {
        tag.try_into()
    }

    /// Returns the tag.
    pub fn tag(&self) -> &[u8] {
        &self.tag
    }
}

impl Packable for IndexationFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + self.tag.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.tag.len() as u8).pack(writer)?;
        writer.write_all(&self.tag)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let tag_length = u8::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK {
            validate_length(tag_length)?;
        }

        let mut tag = vec![0u8; tag_length];
        reader.read_exact(&mut tag)?;

        Ok(Self { tag: tag.into() })
    }
}

#[inline]
fn validate_length(tag_length: usize) -> Result<(), Error> {
    if tag_length == 0 || tag_length > IndexationFeatureBlock::LENGTH_MAX {
        return Err(Error::InvalidIndexationIndexLength(tag_length));
    }

    Ok(())
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable as OldPackable, Read, Write};
use bee_packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix};

pub(crate) type IndexationFeatureBlockLength = BoundedU8<0, { IndexationFeatureBlock::LENGTH_MAX }>;

/// Defines an indexation tag to which the output will be indexed.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, bee_packable::Packable)]
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
    /// Maximum possible length in bytes of an indexation tag.
    pub const LENGTH_MAX: u8 = 64;

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

impl OldPackable for IndexationFeatureBlock {
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
        let tag_length = u8::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK {
            validate_length(tag_length)?;
        }

        let mut tag = vec![0u8; tag_length];
        reader.read_exact(&mut tag)?;

        Self::new(tag)
    }
}

#[inline]
fn validate_length(tag_length: usize) -> Result<(), Error> {
    IndexationFeatureBlockLength::try_from(tag_length).map_err(Error::InvalidIndexationFeatureBlockLength)?;

    Ok(())
}

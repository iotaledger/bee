// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod issuer;
mod metadata;
mod sender;
mod tag;

pub use issuer::IssuerFeatureBlock;
pub use metadata::MetadataFeatureBlock;
pub(crate) use metadata::MetadataFeatureBlockLength;
pub use sender::SenderFeatureBlock;
pub use tag::TagFeatureBlock;
pub(crate) use tag::TagFeatureBlockLength;

use crate::Error;

use bee_common::ord::is_unique_sorted;

use bitflags::bitflags;
use derive_more::{Deref, From};
use packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix, Packable};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidFeatureBlockKind)]
pub enum FeatureBlock {
    /// A sender feature block.
    #[packable(tag = SenderFeatureBlock::KIND)]
    Sender(SenderFeatureBlock),
    /// An issuer feature block.
    #[packable(tag = IssuerFeatureBlock::KIND)]
    Issuer(IssuerFeatureBlock),
    /// A metadata feature block.
    #[packable(tag = MetadataFeatureBlock::KIND)]
    Metadata(MetadataFeatureBlock),
    /// A tag feature block.
    #[packable(tag = TagFeatureBlock::KIND)]
    Tag(TagFeatureBlock),
}

impl FeatureBlock {
    /// Return the output kind of an `Output`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Sender(_) => SenderFeatureBlock::KIND,
            Self::Issuer(_) => IssuerFeatureBlock::KIND,
            Self::Metadata(_) => MetadataFeatureBlock::KIND,
            Self::Tag(_) => TagFeatureBlock::KIND,
        }
    }

    /// Returns the [`FeatureBlockFlags`] for the given [`FeatureBlock`].
    pub(crate) fn flag(&self) -> FeatureBlockFlags {
        match self {
            Self::Sender(_) => FeatureBlockFlags::SENDER,
            Self::Issuer(_) => FeatureBlockFlags::ISSUER,
            Self::Metadata(_) => FeatureBlockFlags::METADATA,
            Self::Tag(_) => FeatureBlockFlags::TAG,
        }
    }
}

pub(crate) type FeatureBlockCount = BoundedU8<0, { FeatureBlocks::COUNT_MAX }>;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deref, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| e.unwrap_packable_or_else(|p| Error::InvalidFeatureBlockCount(p.into())))]
pub struct FeatureBlocks(
    #[packable(verify_with = Self::validate_feature_blocks)] BoxedSlicePrefix<FeatureBlock, FeatureBlockCount>,
);

impl TryFrom<Vec<FeatureBlock>> for FeatureBlocks {
    type Error = Error;

    #[inline(always)]
    fn try_from(feature_blocks: Vec<FeatureBlock>) -> Result<Self, Self::Error> {
        Self::new(feature_blocks)
    }
}

impl FeatureBlocks {
    ///
    pub const COUNT_MAX: u8 = 4;

    /// Creates a new `FeatureBlocks`.
    pub fn new(feature_blocks: Vec<FeatureBlock>) -> Result<Self, Error> {
        let mut feature_blocks =
            BoxedSlicePrefix::<FeatureBlock, FeatureBlockCount>::try_from(feature_blocks.into_boxed_slice())
                .map_err(Error::InvalidFeatureBlockCount)?;

        feature_blocks.sort_by_key(FeatureBlock::kind);
        Self::validate_feature_blocks::<true>(&feature_blocks)?;

        Ok(Self(feature_blocks))
    }

    fn validate_feature_blocks<const VERIFY: bool>(feature_blocks: &[FeatureBlock]) -> Result<(), Error> {
        if VERIFY {
            // Sort is obviously fine now but uniqueness still needs to be checked.
            validate_unique_sorted(feature_blocks)
        } else {
            Ok(())
        }
    }

    /// Gets a reference to a feature block from a feature block kind, if found.
    #[inline(always)]
    pub fn get(&self, key: u8) -> Option<&FeatureBlock> {
        self.0
            .binary_search_by_key(&key, FeatureBlock::kind)
            // SAFETY: indexation is fine since the index has been found.
            .map(|index| &self.0[index])
            .ok()
    }

    /// Returns the length of the feature blocks.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the [`FeatureBlocks`] is empty or not.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[inline]
fn validate_unique_sorted(feature_blocks: &[FeatureBlock]) -> Result<(), Error> {
    if !is_unique_sorted(feature_blocks.iter().map(FeatureBlock::kind)) {
        return Err(Error::FeatureBlocksNotUniqueSorted);
    }

    Ok(())
}

pub(crate) fn validate_allowed_feature_blocks(
    feature_blocks: &FeatureBlocks,
    allowed_feature_blocks: FeatureBlockFlags,
) -> Result<(), Error> {
    for (index, feature_block) in feature_blocks.iter().enumerate() {
        if !allowed_feature_blocks.contains(feature_block.flag()) {
            return Err(Error::UnallowedFeatureBlock {
                index,
                kind: feature_block.kind(),
            });
        }
    }
    Ok(())
}

bitflags! {
    /// A bitflags-based representation of the set of active feature blocks.
    pub(crate) struct FeatureBlockFlags: u16 {
        /// Signals the presence of a [`SenderFeatureBlock`].
        const SENDER = 1 << SenderFeatureBlock::KIND;
        /// Signals the presence of a [`IssuerFeatureBlock`].
        const ISSUER = 1 << IssuerFeatureBlock::KIND;
        /// Signals the presence of a [`MetadataFeatureBlock`].
        const METADATA = 1 << MetadataFeatureBlock::KIND;
        /// Signals the presence of a [`TagFeatureBlock`].
        const TAG = 1 << TagFeatureBlock::KIND;
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod dust_deposit_return;
mod expiration_milestone_index;
mod expiration_unix;
mod indexation;
mod issuer;
mod metadata;
mod sender;
mod timelock_milestone_index;
mod timelock_unix;

pub use dust_deposit_return::DustDepositReturnFeatureBlock;
pub use expiration_milestone_index::ExpirationMilestoneIndexFeatureBlock;
pub use expiration_unix::ExpirationUnixFeatureBlock;
pub use indexation::IndexationFeatureBlock;
pub use issuer::IssuerFeatureBlock;
pub use metadata::MetadataFeatureBlock;
pub use sender::SenderFeatureBlock;
pub use timelock_milestone_index::TimelockMilestoneIndexFeatureBlock;
pub use timelock_unix::TimelockUnixFeatureBlock;

use crate::Error;

use bee_common::{
    ord::is_unique_sorted,
    packable::{Packable, Read, Write},
};

use core::ops::Deref;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, derive_more::From)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum FeatureBlock {
    /// A sender feature block.
    Sender(SenderFeatureBlock),
    /// An issuer feature block.
    Issuer(IssuerFeatureBlock),
    /// A dust deposit return feature block.
    DustDepositReturn(DustDepositReturnFeatureBlock),
    /// A timelock milestone index feature block.
    TimelockMilestoneIndex(TimelockMilestoneIndexFeatureBlock),
    /// A timelock unix feature block.
    TimelockUnix(TimelockUnixFeatureBlock),
    /// An expiration milestone index feature block.
    ExpirationMilestoneIndex(ExpirationMilestoneIndexFeatureBlock),
    /// An expiration unix feature block.
    ExpirationUnix(ExpirationUnixFeatureBlock),
    /// An indexation feature block.
    Indexation(IndexationFeatureBlock),
    /// A metadata feature block.
    Metadata(MetadataFeatureBlock),
}

impl FeatureBlock {
    /// Return the output kind of an `Output`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Sender(_) => SenderFeatureBlock::KIND,
            Self::Issuer(_) => IssuerFeatureBlock::KIND,
            Self::DustDepositReturn(_) => DustDepositReturnFeatureBlock::KIND,
            Self::TimelockMilestoneIndex(_) => TimelockMilestoneIndexFeatureBlock::KIND,
            Self::TimelockUnix(_) => TimelockUnixFeatureBlock::KIND,
            Self::ExpirationMilestoneIndex(_) => ExpirationMilestoneIndexFeatureBlock::KIND,
            Self::ExpirationUnix(_) => ExpirationUnixFeatureBlock::KIND,
            Self::Indexation(_) => IndexationFeatureBlock::KIND,
            Self::Metadata(_) => MetadataFeatureBlock::KIND,
        }
    }
}

impl Packable for FeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Sender(output) => SenderFeatureBlock::KIND.packed_len() + output.packed_len(),
            Self::Issuer(output) => IssuerFeatureBlock::KIND.packed_len() + output.packed_len(),
            Self::DustDepositReturn(output) => DustDepositReturnFeatureBlock::KIND.packed_len() + output.packed_len(),
            Self::TimelockMilestoneIndex(output) => {
                TimelockMilestoneIndexFeatureBlock::KIND.packed_len() + output.packed_len()
            }
            Self::TimelockUnix(output) => TimelockUnixFeatureBlock::KIND.packed_len() + output.packed_len(),
            Self::ExpirationMilestoneIndex(output) => {
                ExpirationMilestoneIndexFeatureBlock::KIND.packed_len() + output.packed_len()
            }
            Self::ExpirationUnix(output) => ExpirationUnixFeatureBlock::KIND.packed_len() + output.packed_len(),
            Self::Indexation(output) => IndexationFeatureBlock::KIND.packed_len() + output.packed_len(),
            Self::Metadata(output) => MetadataFeatureBlock::KIND.packed_len() + output.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Sender(output) => {
                SenderFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Issuer(output) => {
                IssuerFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::DustDepositReturn(output) => {
                DustDepositReturnFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::TimelockMilestoneIndex(output) => {
                TimelockMilestoneIndexFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::TimelockUnix(output) => {
                TimelockUnixFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::ExpirationMilestoneIndex(output) => {
                ExpirationMilestoneIndexFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::ExpirationUnix(output) => {
                ExpirationUnixFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Indexation(output) => {
                IndexationFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Metadata(output) => {
                MetadataFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            SenderFeatureBlock::KIND => SenderFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into(),
            IssuerFeatureBlock::KIND => IssuerFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into(),
            DustDepositReturnFeatureBlock::KIND => {
                DustDepositReturnFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into()
            }
            TimelockMilestoneIndexFeatureBlock::KIND => {
                TimelockMilestoneIndexFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into()
            }
            TimelockUnixFeatureBlock::KIND => TimelockUnixFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into(),
            ExpirationMilestoneIndexFeatureBlock::KIND => {
                ExpirationMilestoneIndexFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into()
            }
            ExpirationUnixFeatureBlock::KIND => ExpirationUnixFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into(),
            IndexationFeatureBlock::KIND => ExpirationUnixFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into(),
            MetadataFeatureBlock::KIND => MetadataFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidFeatureBlockKind(k)),
        })
    }
}

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct FeatureBlocks(Box<[FeatureBlock]>);

impl TryFrom<Vec<FeatureBlock>> for FeatureBlocks {
    type Error = Error;

    fn try_from(mut feature_blocks: Vec<FeatureBlock>) -> Result<Self, Self::Error> {
        validate_count(feature_blocks.len())?;

        feature_blocks.sort_by_key(FeatureBlock::kind);

        // Sort is obviously fine now but uniqueness still needs to be checked.
        validate_unique_sorted(&feature_blocks)?;
        validate_dependencies(&feature_blocks)?;

        Ok(Self(feature_blocks.into_boxed_slice()))
    }
}

impl FeatureBlocks {
    ///
    pub const COUNT_MAX: usize = 8;

    /// Creates a new `FeatureBlocks`.
    pub fn new(feature_blocks: Vec<FeatureBlock>) -> Result<Self, Error> {
        Self::try_from(feature_blocks)
    }

    /// Gets a reference to a feature block from a feature block kind, if found.
    pub fn get(&self, key: u8) -> Option<&FeatureBlock> {
        self.0
            .binary_search_by_key(&key, FeatureBlock::kind)
            // SAFETY: indexation is fine since the index has been found.
            .map(|index| &self.0[index])
            .ok()
    }

    /// Returns the length of the feature blocks.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Deref for FeatureBlocks {
    type Target = [FeatureBlock];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Packable for FeatureBlocks {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + self.0.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.0.len() as u8).pack(writer)?;
        for feature_block in self.0.iter() {
            feature_block.pack(writer)?
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let feature_blocks_count = u8::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK {
            validate_count(feature_blocks_count)?;
        }

        let mut feature_blocks = Vec::with_capacity(feature_blocks_count);
        for _ in 0..feature_blocks_count {
            feature_blocks.push(FeatureBlock::unpack_inner::<R, CHECK>(reader)?);
        }

        if CHECK {
            validate_unique_sorted(&feature_blocks)?;
            validate_dependencies(&feature_blocks)?;
        };

        Ok(Self(feature_blocks.into_boxed_slice()))
    }
}

#[inline]
fn validate_count(feature_blocks_count: usize) -> Result<(), Error> {
    if feature_blocks_count > FeatureBlocks::COUNT_MAX {
        return Err(Error::InvalidFeatureBlockCount(feature_blocks_count));
    }

    Ok(())
}

#[inline]
fn validate_unique_sorted(feature_blocks: &[FeatureBlock]) -> Result<(), Error> {
    if !is_unique_sorted(feature_blocks.iter().map(FeatureBlock::kind)) {
        return Err(Error::FeatureBlocksNotUniqueSorted);
    }

    Ok(())
}

#[inline]
fn validate_dependencies(feature_blocks: &[FeatureBlock]) -> Result<(), Error> {
    if (feature_blocks
        .binary_search_by_key(&ExpirationMilestoneIndexFeatureBlock::KIND, FeatureBlock::kind)
        .is_ok()
        || feature_blocks
            .binary_search_by_key(&ExpirationUnixFeatureBlock::KIND, FeatureBlock::kind)
            .is_ok())
        && feature_blocks
            .binary_search_by_key(&SenderFeatureBlock::KIND, FeatureBlock::kind)
            .is_err()
    {
        return Err(Error::ExpirationUnixFeatureBlockWithoutSenderFeatureBlock);
    }

    Ok(())
}

pub(crate) fn validate_allowed_feature_blocks(
    feature_blocks: &FeatureBlocks,
    allowed_feature_blocks_kind: &[u8],
) -> Result<(), Error> {
    if feature_blocks.len() > allowed_feature_blocks_kind.len() {
        return Err(Error::TooManyFeatureBlocks(
            allowed_feature_blocks_kind.len(),
            feature_blocks.len(),
        ));
    }

    let mut index = 0;

    for feature_block in feature_blocks.iter() {
        index = allowed_feature_blocks_kind[index..]
            .binary_search(&feature_block.kind())
            .map_err(|index| Error::UnallowedFeatureBlock(index, feature_block.kind()))?;
    }

    Ok(())
}

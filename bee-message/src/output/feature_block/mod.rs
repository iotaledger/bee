// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod expiration_milestone_index;
mod expiration_unix;
mod indexation;
mod issuer;
mod metadata;
mod return_amount;
mod sender;
mod timelock_milestone_index;
mod timelock_unix;

pub use expiration_milestone_index::ExpirationMilestoneIndexFeatureBlock;
pub use expiration_unix::ExpirationUnixFeatureBlock;
pub use indexation::IndexationFeatureBlock;
pub use issuer::IssuerFeatureBlock;
pub use metadata::MetadataFeatureBlock;
pub use return_amount::ReturnAmountFeatureBlock;
pub use sender::SenderFeatureBlock;
pub use timelock_milestone_index::TimelockMilestoneIndexFeatureBlock;
pub use timelock_unix::TimelockUnixFeatureBlock;

use crate::Error;

use bee_common::{
    ord::is_unique_sorted,
    packable::{Packable, Read, Write},
};

use core::{convert::TryFrom, ops::Deref};

///
pub const FEATURE_BLOCK_COUNT_MAX: u16 = 8;

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
    /// A return amount feature block.
    ReturnAmount(ReturnAmountFeatureBlock),
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
            Self::ReturnAmount(_) => ReturnAmountFeatureBlock::KIND,
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
            Self::ReturnAmount(output) => ReturnAmountFeatureBlock::KIND.packed_len() + output.packed_len(),
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
            Self::ReturnAmount(output) => {
                ReturnAmountFeatureBlock::KIND.pack(writer)?;
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
            ReturnAmountFeatureBlock::KIND => ReturnAmountFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into(),
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

    fn try_from(feature_blocks: Vec<FeatureBlock>) -> Result<Self, Self::Error> {
        if feature_blocks.len() as u16 > FEATURE_BLOCK_COUNT_MAX {
            return Err(Error::InvalidFeatureBlockCount(feature_blocks.len() as u16));
        }

        if !is_unique_sorted(feature_blocks.iter().map(|b| b.kind())) {
            return Err(Error::FeatureBlocksNotUniqueSorted);
        }

        Ok(Self(feature_blocks.into_boxed_slice()))
    }
}

impl FeatureBlocks {
    /// Creates a new `FeatureBlocks`.
    pub fn new(feature_blocks: Vec<FeatureBlock>) -> Result<Self, Error> {
        Self::try_from(feature_blocks)
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
        0u16.packed_len() + self.0.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.0.len() as u16).pack(writer)?;
        for feature_block in self.0.iter() {
            feature_block.pack(writer)?
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let feature_blocks_len = u16::unpack_inner::<R, CHECK>(reader)?;

        if CHECK && feature_blocks_len > FEATURE_BLOCK_COUNT_MAX {
            return Err(Error::InvalidFeatureBlockCount(feature_blocks_len));
        }

        let mut feature_blocks = Vec::with_capacity(feature_blocks_len as usize);
        for _ in 0..feature_blocks_len {
            feature_blocks.push(FeatureBlock::unpack_inner::<R, CHECK>(reader)?);
        }

        Self::new(feature_blocks)
    }
}

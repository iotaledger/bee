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

pub(crate) use dust_deposit_return::DustDepositAmount;
pub use dust_deposit_return::DustDepositReturnFeatureBlock;
pub use expiration_milestone_index::ExpirationMilestoneIndexFeatureBlock;
pub use expiration_unix::ExpirationUnixFeatureBlock;
pub use indexation::IndexationFeatureBlock;
pub(crate) use indexation::IndexationFeatureBlockLength;
pub use issuer::IssuerFeatureBlock;
pub use metadata::MetadataFeatureBlock;
pub(crate) use metadata::MetadataFeatureBlockLength;
pub use sender::SenderFeatureBlock;
pub use timelock_milestone_index::TimelockMilestoneIndexFeatureBlock;
pub use timelock_unix::TimelockUnixFeatureBlock;

use crate::Error;

use bee_common::{
    ord::is_unique_sorted,
    packable::{Packable as OldPackable, Read, Write},
};
use bee_packable::{
    bounded::BoundedU8,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::BoxedSlicePrefix,
};

use bitflags::bitflags;
use derive_more::{Deref, From};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, bee_packable::Packable)]
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
    /// A dust deposit return feature block.
    #[packable(tag = DustDepositReturnFeatureBlock::KIND)]
    DustDepositReturn(DustDepositReturnFeatureBlock),
    /// A timelock milestone index feature block.
    #[packable(tag = TimelockMilestoneIndexFeatureBlock::KIND)]
    TimelockMilestoneIndex(TimelockMilestoneIndexFeatureBlock),
    /// A timelock unix feature block.
    #[packable(tag = TimelockUnixFeatureBlock::KIND)]
    TimelockUnix(TimelockUnixFeatureBlock),
    /// An expiration milestone index feature block.
    #[packable(tag = ExpirationMilestoneIndexFeatureBlock::KIND)]
    ExpirationMilestoneIndex(ExpirationMilestoneIndexFeatureBlock),
    /// An expiration unix feature block.
    #[packable(tag = ExpirationUnixFeatureBlock::KIND)]
    ExpirationUnix(ExpirationUnixFeatureBlock),
    /// A metadata feature block.
    #[packable(tag = MetadataFeatureBlock::KIND)]
    Metadata(MetadataFeatureBlock),
    /// An indexation feature block.
    #[packable(tag = IndexationFeatureBlock::KIND)]
    Indexation(IndexationFeatureBlock),
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
            Self::Metadata(_) => MetadataFeatureBlock::KIND,
            Self::Indexation(_) => IndexationFeatureBlock::KIND,
        }
    }

    /// Returns the [`FeatureBlockFlags`] for the given [`FeatureBlock`].
    pub(crate) fn flag(&self) -> FeatureBlockFlags {
        match self {
            Self::Sender(_) => FeatureBlockFlags::SENDER,
            Self::Issuer(_) => FeatureBlockFlags::ISSUER,
            Self::DustDepositReturn(_) => FeatureBlockFlags::DUST_DEPOSIT_RETURN,
            Self::TimelockMilestoneIndex(_) => FeatureBlockFlags::TIMELOCK_MILESTONE_INDEX,
            Self::TimelockUnix(_) => FeatureBlockFlags::TIMELOCK_UNIX,
            Self::ExpirationMilestoneIndex(_) => FeatureBlockFlags::EXPIRATION_MILESTONE_INDEX,
            Self::ExpirationUnix(_) => FeatureBlockFlags::EXPIRATION_UNIX,
            Self::Metadata(_) => FeatureBlockFlags::METADATA,
            Self::Indexation(_) => FeatureBlockFlags::INDEXATION,
        }
    }
}

impl OldPackable for FeatureBlock {
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
            Self::Metadata(output) => MetadataFeatureBlock::KIND.packed_len() + output.packed_len(),
            Self::Indexation(output) => IndexationFeatureBlock::KIND.packed_len() + output.packed_len(),
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
            Self::Metadata(output) => {
                MetadataFeatureBlock::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Indexation(output) => {
                IndexationFeatureBlock::KIND.pack(writer)?;
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
            MetadataFeatureBlock::KIND => MetadataFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into(),
            IndexationFeatureBlock::KIND => IndexationFeatureBlock::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidFeatureBlockKind(k)),
        })
    }
}

pub(crate) type FeatureBlockCount = BoundedU8<0, { FeatureBlocks::COUNT_MAX }>;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deref)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct FeatureBlocks(BoxedSlicePrefix<FeatureBlock, FeatureBlockCount>);

impl TryFrom<Vec<FeatureBlock>> for FeatureBlocks {
    type Error = Error;

    #[inline(always)]
    fn try_from(feature_blocks: Vec<FeatureBlock>) -> Result<Self, Self::Error> {
        Self::new(feature_blocks)
    }
}

impl FeatureBlocks {
    ///
    pub const COUNT_MAX: u8 = 9;

    /// Creates a new `FeatureBlocks`.
    pub fn new(feature_blocks: Vec<FeatureBlock>) -> Result<Self, Error> {
        let mut feature_blocks =
            BoxedSlicePrefix::<FeatureBlock, FeatureBlockCount>::try_from(feature_blocks.into_boxed_slice())
                .map_err(Error::InvalidFeatureBlockCount)?;

        feature_blocks.sort_by_key(FeatureBlock::kind);
        // Sort is obviously fine now but uniqueness still needs to be checked.
        Self::from_boxed_slice::<true>(feature_blocks)
    }

    fn from_boxed_slice<const VERIFY: bool>(
        feature_blocks: BoxedSlicePrefix<FeatureBlock, FeatureBlockCount>,
    ) -> Result<Self, Error> {
        if VERIFY {
            validate_unique_sorted(&feature_blocks)?;
            validate_dependencies(&feature_blocks)?;
        }

        Ok(Self(feature_blocks))
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

impl bee_packable::Packable for FeatureBlocks {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.0.pack(packer)
    }

    fn unpack<U: bee_packable::unpacker::Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let feature_blocks = BoxedSlicePrefix::<FeatureBlock, FeatureBlockCount>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| {
                err.unwrap_packable_or_else(|prefix_err| Error::InvalidFeatureBlockCount(prefix_err.into()))
            })?;

        Self::from_boxed_slice::<VERIFY>(feature_blocks).map_err(UnpackError::Packable)
    }
}

impl OldPackable for FeatureBlocks {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + self.0.iter().map(OldPackable::packed_len).sum::<usize>()
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

        Self::new(feature_blocks)
    }
}

#[inline]
fn validate_count(feature_blocks_count: usize) -> Result<(), Error> {
    FeatureBlockCount::try_from(feature_blocks_count).map_err(Error::InvalidFeatureBlockCount)?;

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
        .binary_search_by_key(&DustDepositReturnFeatureBlock::KIND, FeatureBlock::kind)
        .is_ok()
        || feature_blocks
            .binary_search_by_key(&ExpirationMilestoneIndexFeatureBlock::KIND, FeatureBlock::kind)
            .is_ok()
        || feature_blocks
            .binary_search_by_key(&ExpirationUnixFeatureBlock::KIND, FeatureBlock::kind)
            .is_ok())
        && feature_blocks
            .binary_search_by_key(&SenderFeatureBlock::KIND, FeatureBlock::kind)
            .is_err()
    {
        return Err(Error::MissingRequiredSenderBlock);
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
        /// Signals the presence of a [`DustDepositReturnFeatureBlock`].
        const DUST_DEPOSIT_RETURN = 1 << DustDepositReturnFeatureBlock::KIND;
        /// Signals the presence of a [`TimelockMilestoneIndexFeatureBlock`].
        const TIMELOCK_MILESTONE_INDEX = 1 << TimelockMilestoneIndexFeatureBlock::KIND;
        /// Signals the presence of a [`TimelockUnixFeatureBlock`].
        const TIMELOCK_UNIX = 1 << TimelockUnixFeatureBlock::KIND;
        /// Signals the presence of a [`ExpirationMilestoneIndexFeatureBlock`].
        const EXPIRATION_MILESTONE_INDEX = 1 << ExpirationMilestoneIndexFeatureBlock::KIND;
        /// Signals the presence of a [`ExpirationUnixFeatureBlock`].
        const EXPIRATION_UNIX = 1 << ExpirationUnixFeatureBlock::KIND;
        /// Signals the presence of a [`MetadataFeatureBlock`].
        const METADATA = 1 << MetadataFeatureBlock::KIND;
        /// Signals the presence of a [`IndexationFeatureBlock`].
        const INDEXATION = 1 << IndexationFeatureBlock::KIND;
    }
}

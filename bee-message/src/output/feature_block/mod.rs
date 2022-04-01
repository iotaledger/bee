// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod issuer;
mod metadata;
mod sender;
mod tag;

use alloc::vec::Vec;

use bitflags::bitflags;
use derive_more::{Deref, From};
use iterator_sorted::is_unique_sorted;
use packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix, Packable};

pub use self::{
    issuer::IssuerFeatureBlock, metadata::MetadataFeatureBlock, sender::SenderFeatureBlock, tag::TagFeatureBlock,
};
pub(crate) use self::{metadata::MetadataFeatureBlockLength, tag::TagFeatureBlockLength};
use crate::{create_bitflags, Error};

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
    pub fn flag(&self) -> FeatureBlockFlags {
        match self {
            Self::Sender(_) => FeatureBlockFlags::SENDER,
            Self::Issuer(_) => FeatureBlockFlags::ISSUER,
            Self::Metadata(_) => FeatureBlockFlags::METADATA,
            Self::Tag(_) => FeatureBlockFlags::TAG,
        }
    }
}

create_bitflags!(
    /// A bitflags-based representation of the set of active [`FeatureBlock`]s.
    pub FeatureBlockFlags,
    u16,
    [
        (SENDER, SenderFeatureBlock),
        (ISSUER, IssuerFeatureBlock),
        (METADATA, MetadataFeatureBlock),
        (TAG, TagFeatureBlock),
    ]
);

pub(crate) type FeatureBlockCount = BoundedU8<0, { FeatureBlocks::COUNT_MAX }>;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deref, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidFeatureBlockCount(p.into())))]
pub struct FeatureBlocks(
    #[packable(verify_with = verify_unique_sorted)] BoxedSlicePrefix<FeatureBlock, FeatureBlockCount>,
);

impl TryFrom<Vec<FeatureBlock>> for FeatureBlocks {
    type Error = Error;

    #[inline(always)]
    fn try_from(feature_blocks: Vec<FeatureBlock>) -> Result<Self, Self::Error> {
        Self::new(feature_blocks)
    }
}

impl IntoIterator for FeatureBlocks {
    type Item = FeatureBlock;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Vec::from(Into::<Box<[FeatureBlock]>>::into(self.0)).into_iter()
    }
}

impl FeatureBlocks {
    ///
    pub const COUNT_MAX: u8 = 4;

    /// Creates a new [`FeatureBlocks`].
    pub fn new(feature_blocks: Vec<FeatureBlock>) -> Result<Self, Error> {
        let mut feature_blocks =
            BoxedSlicePrefix::<FeatureBlock, FeatureBlockCount>::try_from(feature_blocks.into_boxed_slice())
                .map_err(Error::InvalidFeatureBlockCount)?;

        feature_blocks.sort_by_key(FeatureBlock::kind);
        // Sort is obviously fine now but uniqueness still needs to be checked.
        verify_unique_sorted::<true>(&feature_blocks)?;

        Ok(Self(feature_blocks))
    }

    /// Gets a reference to a [`FeatureBlock`] from a feature block kind, if any.
    #[inline(always)]
    pub fn get(&self, key: u8) -> Option<&FeatureBlock> {
        self.0
            .binary_search_by_key(&key, FeatureBlock::kind)
            // PANIC: indexation is fine since the index has been found.
            .map(|index| &self.0[index])
            .ok()
    }

    /// Gets a reference to a [`SenderFeatureBlock`], if any.
    pub fn sender(&self) -> Option<&SenderFeatureBlock> {
        if let Some(FeatureBlock::Sender(sender)) = self.get(SenderFeatureBlock::KIND) {
            Some(sender)
        } else {
            None
        }
    }

    /// Gets a reference to a [`IssuerFeatureBlock`], if any.
    pub fn issuer(&self) -> Option<&IssuerFeatureBlock> {
        if let Some(FeatureBlock::Issuer(issuer)) = self.get(IssuerFeatureBlock::KIND) {
            Some(issuer)
        } else {
            None
        }
    }

    /// Gets a reference to a [`MetadataFeatureBlock`], if any.
    pub fn metadata(&self) -> Option<&MetadataFeatureBlock> {
        if let Some(FeatureBlock::Metadata(metadata)) = self.get(MetadataFeatureBlock::KIND) {
            Some(metadata)
        } else {
            None
        }
    }

    /// Gets a reference to a [`TagFeatureBlock`], if any.
    pub fn tag(&self) -> Option<&TagFeatureBlock> {
        if let Some(FeatureBlock::Tag(tag)) = self.get(TagFeatureBlock::KIND) {
            Some(tag)
        } else {
            None
        }
    }
}

#[inline]
fn verify_unique_sorted<const VERIFY: bool>(feature_blocks: &[FeatureBlock]) -> Result<(), Error> {
    if VERIFY && !is_unique_sorted(feature_blocks.iter().map(FeatureBlock::kind)) {
        Err(Error::FeatureBlocksNotUniqueSorted)
    } else {
        Ok(())
    }
}

pub(crate) fn verify_allowed_feature_blocks(
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn all_flags_present() {
        assert_eq!(
            FeatureBlockFlags::ALL_FLAGS,
            &[
                FeatureBlockFlags::SENDER,
                FeatureBlockFlags::ISSUER,
                FeatureBlockFlags::METADATA,
                FeatureBlockFlags::TAG
            ]
        );
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::Value;

    pub use self::{
        issuer::dto::IssuerFeatureBlockDto, metadata::dto::MetadataFeatureBlockDto, sender::dto::SenderFeatureBlockDto,
        tag::dto::TagFeatureBlockDto,
    };
    use super::*;
    use crate::error::dto::DtoError;

    #[derive(Clone, Debug)]
    pub enum FeatureBlockDto {
        /// A sender feature block.
        Sender(SenderFeatureBlockDto),
        /// An issuer feature block.
        Issuer(IssuerFeatureBlockDto),
        /// A metadata feature block.
        Metadata(MetadataFeatureBlockDto),
        /// A tag feature block.
        Tag(TagFeatureBlockDto),
    }

    impl<'de> Deserialize<'de> for FeatureBlockDto {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let value = Value::deserialize(d)?;
            Ok(
                match value
                    .get("type")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| serde::de::Error::custom("invalid feature block type"))? as u8
                {
                    SenderFeatureBlock::KIND => {
                        FeatureBlockDto::Sender(SenderFeatureBlockDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize sender feature block: {}", e))
                        })?)
                    }
                    IssuerFeatureBlock::KIND => {
                        FeatureBlockDto::Issuer(IssuerFeatureBlockDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize issuer feature block: {}", e))
                        })?)
                    }
                    MetadataFeatureBlock::KIND => {
                        FeatureBlockDto::Metadata(MetadataFeatureBlockDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize metadata feature block: {}", e))
                        })?)
                    }
                    TagFeatureBlock::KIND => {
                        FeatureBlockDto::Tag(TagFeatureBlockDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize tag feature block: {}", e))
                        })?)
                    }
                    _ => return Err(serde::de::Error::custom("invalid feature block type")),
                },
            )
        }
    }

    impl Serialize for FeatureBlockDto {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            #[derive(Serialize)]
            #[serde(untagged)]
            enum FeatureBlockDto_<'a> {
                T1(&'a SenderFeatureBlockDto),
                T2(&'a IssuerFeatureBlockDto),
                T3(&'a MetadataFeatureBlockDto),
                T4(&'a TagFeatureBlockDto),
            }
            #[derive(Serialize)]
            struct TypedFeatureBlock<'a> {
                #[serde(flatten)]
                feature_block: FeatureBlockDto_<'a>,
            }
            let feature_block = match self {
                FeatureBlockDto::Sender(o) => TypedFeatureBlock {
                    feature_block: FeatureBlockDto_::T1(o),
                },
                FeatureBlockDto::Issuer(o) => TypedFeatureBlock {
                    feature_block: FeatureBlockDto_::T2(o),
                },
                FeatureBlockDto::Metadata(o) => TypedFeatureBlock {
                    feature_block: FeatureBlockDto_::T3(o),
                },
                FeatureBlockDto::Tag(o) => TypedFeatureBlock {
                    feature_block: FeatureBlockDto_::T4(o),
                },
            };
            feature_block.serialize(serializer)
        }
    }

    impl From<&FeatureBlock> for FeatureBlockDto {
        fn from(value: &FeatureBlock) -> Self {
            match value {
                FeatureBlock::Sender(v) => Self::Sender(SenderFeatureBlockDto {
                    kind: SenderFeatureBlock::KIND,
                    address: v.address().into(),
                }),
                FeatureBlock::Issuer(v) => Self::Issuer(IssuerFeatureBlockDto {
                    kind: IssuerFeatureBlock::KIND,
                    address: v.address().into(),
                }),
                FeatureBlock::Metadata(v) => Self::Metadata(MetadataFeatureBlockDto {
                    kind: MetadataFeatureBlock::KIND,
                    data: v.to_string(),
                }),
                FeatureBlock::Tag(v) => Self::Tag(TagFeatureBlockDto {
                    kind: TagFeatureBlock::KIND,
                    tag: v.to_string(),
                }),
            }
        }
    }

    impl TryFrom<&FeatureBlockDto> for FeatureBlock {
        type Error = DtoError;

        fn try_from(value: &FeatureBlockDto) -> Result<Self, Self::Error> {
            Ok(match value {
                FeatureBlockDto::Sender(v) => Self::Sender(SenderFeatureBlock::new((&v.address).try_into()?)),
                FeatureBlockDto::Issuer(v) => Self::Issuer(IssuerFeatureBlock::new((&v.address).try_into()?)),
                FeatureBlockDto::Metadata(v) => Self::Metadata(MetadataFeatureBlock::new(
                    prefix_hex::decode(&v.data).map_err(|_e| DtoError::InvalidField("MetadataFeatureBlock"))?,
                )?),
                FeatureBlockDto::Tag(v) => Self::Tag(TagFeatureBlock::new(
                    prefix_hex::decode(&v.tag).map_err(|_e| DtoError::InvalidField("TagFeatureBlock"))?,
                )?),
            })
        }
    }

    impl FeatureBlockDto {
        /// Return the feature block kind of a `FeatureBlockDto`.
        pub fn kind(&self) -> u8 {
            match self {
                Self::Sender(_) => SenderFeatureBlock::KIND,
                Self::Issuer(_) => IssuerFeatureBlock::KIND,
                Self::Metadata(_) => MetadataFeatureBlock::KIND,
                Self::Tag(_) => TagFeatureBlock::KIND,
            }
        }
    }
}

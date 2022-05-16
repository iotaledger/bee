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

pub use self::{issuer::IssuerFeature, metadata::MetadataFeature, sender::SenderFeature, tag::TagFeature};
pub(crate) use self::{metadata::MetadataFeatureLength, tag::TagFeatureLength};
use crate::{create_bitflags, Error};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidFeatureKind)]
pub enum Feature {
    /// A sender feature.
    #[packable(tag = SenderFeature::KIND)]
    Sender(SenderFeature),
    /// An issuer feature.
    #[packable(tag = IssuerFeature::KIND)]
    Issuer(IssuerFeature),
    /// A metadata feature.
    #[packable(tag = MetadataFeature::KIND)]
    Metadata(MetadataFeature),
    /// A tag feature.
    #[packable(tag = TagFeature::KIND)]
    Tag(TagFeature),
}

impl Feature {
    /// Return the output kind of an `Output`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Sender(_) => SenderFeature::KIND,
            Self::Issuer(_) => IssuerFeature::KIND,
            Self::Metadata(_) => MetadataFeature::KIND,
            Self::Tag(_) => TagFeature::KIND,
        }
    }

    /// Returns the [`FeatureFlags`] for the given [`Feature`].
    pub fn flag(&self) -> FeatureFlags {
        match self {
            Self::Sender(_) => FeatureFlags::SENDER,
            Self::Issuer(_) => FeatureFlags::ISSUER,
            Self::Metadata(_) => FeatureFlags::METADATA,
            Self::Tag(_) => FeatureFlags::TAG,
        }
    }
}

create_bitflags!(
    /// A bitflags-based representation of the set of active [`Feature`]s.
    pub FeatureFlags,
    u16,
    [
        (SENDER, SenderFeature),
        (ISSUER, IssuerFeature),
        (METADATA, MetadataFeature),
        (TAG, TagFeature),
    ]
);

pub(crate) type FeatureCount = BoundedU8<0, { Features::COUNT_MAX }>;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deref, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidFeatureCount(p.into())))]
pub struct Features(#[packable(verify_with = verify_unique_sorted)] BoxedSlicePrefix<Feature, FeatureCount>);

impl TryFrom<Vec<Feature>> for Features {
    type Error = Error;

    #[inline(always)]
    fn try_from(features: Vec<Feature>) -> Result<Self, Self::Error> {
        Self::new(features)
    }
}

impl IntoIterator for Features {
    type Item = Feature;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Vec::from(Into::<Box<[Feature]>>::into(self.0)).into_iter()
    }
}

impl Features {
    ///
    pub const COUNT_MAX: u8 = 4;

    /// Creates a new [`Features`].
    pub fn new(features: Vec<Feature>) -> Result<Self, Error> {
        let mut features = BoxedSlicePrefix::<Feature, FeatureCount>::try_from(features.into_boxed_slice())
            .map_err(Error::InvalidFeatureCount)?;

        features.sort_by_key(Feature::kind);
        // Sort is obviously fine now but uniqueness still needs to be checked.
        verify_unique_sorted::<true>(&features)?;

        Ok(Self(features))
    }

    /// Gets a reference to a [`Feature`] from a feature kind, if any.
    #[inline(always)]
    pub fn get(&self, key: u8) -> Option<&Feature> {
        self.0
            .binary_search_by_key(&key, Feature::kind)
            // PANIC: indexation is fine since the index has been found.
            .map(|index| &self.0[index])
            .ok()
    }

    /// Gets a reference to a [`SenderFeature`], if any.
    pub fn sender(&self) -> Option<&SenderFeature> {
        if let Some(Feature::Sender(sender)) = self.get(SenderFeature::KIND) {
            Some(sender)
        } else {
            None
        }
    }

    /// Gets a reference to a [`IssuerFeature`], if any.
    pub fn issuer(&self) -> Option<&IssuerFeature> {
        if let Some(Feature::Issuer(issuer)) = self.get(IssuerFeature::KIND) {
            Some(issuer)
        } else {
            None
        }
    }

    /// Gets a reference to a [`MetadataFeature`], if any.
    pub fn metadata(&self) -> Option<&MetadataFeature> {
        if let Some(Feature::Metadata(metadata)) = self.get(MetadataFeature::KIND) {
            Some(metadata)
        } else {
            None
        }
    }

    /// Gets a reference to a [`TagFeature`], if any.
    pub fn tag(&self) -> Option<&TagFeature> {
        if let Some(Feature::Tag(tag)) = self.get(TagFeature::KIND) {
            Some(tag)
        } else {
            None
        }
    }
}

#[inline]
fn verify_unique_sorted<const VERIFY: bool>(features: &[Feature]) -> Result<(), Error> {
    if VERIFY && !is_unique_sorted(features.iter().map(Feature::kind)) {
        Err(Error::FeaturesNotUniqueSorted)
    } else {
        Ok(())
    }
}

pub(crate) fn verify_allowed_features(features: &Features, allowed_features: FeatureFlags) -> Result<(), Error> {
    for (index, feature) in features.iter().enumerate() {
        if !allowed_features.contains(feature.flag()) {
            return Err(Error::UnallowedFeature {
                index,
                kind: feature.kind(),
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
            FeatureFlags::ALL_FLAGS,
            &[
                FeatureFlags::SENDER,
                FeatureFlags::ISSUER,
                FeatureFlags::METADATA,
                FeatureFlags::TAG
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
        issuer::dto::IssuerFeatureDto, metadata::dto::MetadataFeatureDto, sender::dto::SenderFeatureDto,
        tag::dto::TagFeatureDto,
    };
    use super::*;
    use crate::error::dto::DtoError;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum FeatureDto {
        /// A sender feature.
        Sender(SenderFeatureDto),
        /// An issuer feature.
        Issuer(IssuerFeatureDto),
        /// A metadata feature.
        Metadata(MetadataFeatureDto),
        /// A tag feature.
        Tag(TagFeatureDto),
    }

    impl<'de> Deserialize<'de> for FeatureDto {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let value = Value::deserialize(d)?;
            Ok(
                match value
                    .get("type")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| serde::de::Error::custom("invalid feature type"))? as u8
                {
                    SenderFeature::KIND => {
                        FeatureDto::Sender(SenderFeatureDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize sender feature: {}", e))
                        })?)
                    }
                    IssuerFeature::KIND => {
                        FeatureDto::Issuer(IssuerFeatureDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize issuer feature: {}", e))
                        })?)
                    }
                    MetadataFeature::KIND => {
                        FeatureDto::Metadata(MetadataFeatureDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize metadata feature: {}", e))
                        })?)
                    }
                    TagFeature::KIND => FeatureDto::Tag(
                        TagFeatureDto::deserialize(value)
                            .map_err(|e| serde::de::Error::custom(format!("cannot deserialize tag feature: {}", e)))?,
                    ),
                    _ => return Err(serde::de::Error::custom("invalid feature type")),
                },
            )
        }
    }

    impl Serialize for FeatureDto {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            #[derive(Serialize)]
            #[serde(untagged)]
            enum FeatureDto_<'a> {
                T1(&'a SenderFeatureDto),
                T2(&'a IssuerFeatureDto),
                T3(&'a MetadataFeatureDto),
                T4(&'a TagFeatureDto),
            }
            #[derive(Serialize)]
            struct TypedFeature<'a> {
                #[serde(flatten)]
                feature: FeatureDto_<'a>,
            }
            let feature = match self {
                FeatureDto::Sender(o) => TypedFeature {
                    feature: FeatureDto_::T1(o),
                },
                FeatureDto::Issuer(o) => TypedFeature {
                    feature: FeatureDto_::T2(o),
                },
                FeatureDto::Metadata(o) => TypedFeature {
                    feature: FeatureDto_::T3(o),
                },
                FeatureDto::Tag(o) => TypedFeature {
                    feature: FeatureDto_::T4(o),
                },
            };
            feature.serialize(serializer)
        }
    }

    impl From<&Feature> for FeatureDto {
        fn from(value: &Feature) -> Self {
            match value {
                Feature::Sender(v) => Self::Sender(SenderFeatureDto {
                    kind: SenderFeature::KIND,
                    address: v.address().into(),
                }),
                Feature::Issuer(v) => Self::Issuer(IssuerFeatureDto {
                    kind: IssuerFeature::KIND,
                    address: v.address().into(),
                }),
                Feature::Metadata(v) => Self::Metadata(MetadataFeatureDto {
                    kind: MetadataFeature::KIND,
                    data: v.to_string(),
                }),
                Feature::Tag(v) => Self::Tag(TagFeatureDto {
                    kind: TagFeature::KIND,
                    tag: v.to_string(),
                }),
            }
        }
    }

    impl TryFrom<&FeatureDto> for Feature {
        type Error = DtoError;

        fn try_from(value: &FeatureDto) -> Result<Self, Self::Error> {
            Ok(match value {
                FeatureDto::Sender(v) => Self::Sender(SenderFeature::new((&v.address).try_into()?)),
                FeatureDto::Issuer(v) => Self::Issuer(IssuerFeature::new((&v.address).try_into()?)),
                FeatureDto::Metadata(v) => Self::Metadata(MetadataFeature::new(
                    prefix_hex::decode(&v.data).map_err(|_e| DtoError::InvalidField("MetadataFeature"))?,
                )?),
                FeatureDto::Tag(v) => Self::Tag(TagFeature::new(
                    prefix_hex::decode(&v.tag).map_err(|_e| DtoError::InvalidField("TagFeature"))?,
                )?),
            })
        }
    }

    impl FeatureDto {
        /// Return the feature kind of a `FeatureDto`.
        pub fn kind(&self) -> u8 {
            match self {
                Self::Sender(_) => SenderFeature::KIND,
                Self::Issuer(_) => IssuerFeature::KIND,
                Self::Metadata(_) => MetadataFeature::KIND,
                Self::Tag(_) => TagFeature::KIND,
            }
        }
    }
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::output::feature::{Feature, FeatureFlags, IssuerFeature, MetadataFeature, SenderFeature, TagFeature};

use crate::rand::{address::rand_address, bytes::rand_bytes, number::rand_number_range};

/// Generates a random [`SenderFeature`].
pub fn rand_sender_feature() -> SenderFeature {
    SenderFeature::new(rand_address())
}

/// Generates a random [`IssuerFeature`].
pub fn rand_issuer_feature() -> IssuerFeature {
    IssuerFeature::new(rand_address())
}

/// Generates a random [`MetadataFeature`].
pub fn rand_metadata_feature() -> MetadataFeature {
    let bytes = rand_bytes(rand_number_range(MetadataFeature::LENGTH_RANGE) as usize);
    MetadataFeature::new(bytes).unwrap()
}

/// Generates a random [`TagFeature`].
pub fn rand_tag_feature() -> TagFeature {
    let bytes = rand_bytes(rand_number_range(TagFeature::LENGTH_RANGE) as usize);
    TagFeature::new(bytes).unwrap()
}

fn rand_feature_from_flag(flag: &FeatureFlags) -> Feature {
    match *flag {
        FeatureFlags::SENDER => Feature::Sender(rand_sender_feature()),
        FeatureFlags::ISSUER => Feature::Issuer(rand_issuer_feature()),
        FeatureFlags::METADATA => Feature::Metadata(rand_metadata_feature()),
        FeatureFlags::TAG => Feature::Tag(rand_tag_feature()),
        _ => unreachable!(),
    }
}

/// Generates a [`Vec`] of random [`Feature`]s given a set of allowed [`FeatureFlags`].
pub fn rand_allowed_features(allowed_features: FeatureFlags) -> Vec<Feature> {
    let mut all_features = FeatureFlags::ALL_FLAGS
        .iter()
        .map(rand_feature_from_flag)
        .collect::<Vec<_>>();
    all_features.retain(|feature| allowed_features.contains(feature.flag()));
    all_features
}

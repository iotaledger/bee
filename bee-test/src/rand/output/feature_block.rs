// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{address::rand_address, bytes::rand_bytes, number::rand_number_range};

use bee_message::output::feature_block::{
    FeatureBlock, FeatureBlockFlags, IssuerFeatureBlock, MetadataFeatureBlock, SenderFeatureBlock, TagFeatureBlock,
};

/// Generates a random [`SenderFeatureBlock`].
pub fn rand_sender_feature_block() -> SenderFeatureBlock {
    SenderFeatureBlock::new(rand_address())
}

/// Generates a random [`IssuerFeatureBlock`].
pub fn rand_issuer_feature_block() -> IssuerFeatureBlock {
    IssuerFeatureBlock::new(rand_address())
}

/// Generates a random [`MetadataFeatureBlock`].
pub fn rand_metadata_feature_block() -> MetadataFeatureBlock {
    let bytes = rand_bytes(rand_number_range(MetadataFeatureBlock::LENGTH_RANGE) as usize);
    MetadataFeatureBlock::new(bytes).unwrap()
}

/// Generates a random [`TagFeatureBlock`].
pub fn rand_tag_feature_block() -> TagFeatureBlock {
    let bytes = rand_bytes(rand_number_range(TagFeatureBlock::LENGTH_RANGE) as usize);
    TagFeatureBlock::new(bytes).unwrap()
}

fn rand_feature_block_from_flag(flag: &FeatureBlockFlags) -> FeatureBlock {
    match *flag {
        FeatureBlockFlags::SENDER => FeatureBlock::Sender(rand_sender_feature_block()),
        FeatureBlockFlags::ISSUER => FeatureBlock::Issuer(rand_issuer_feature_block()),
        FeatureBlockFlags::METADATA => FeatureBlock::Metadata(rand_metadata_feature_block()),
        FeatureBlockFlags::TAG => FeatureBlock::Tag(rand_tag_feature_block()),
        _ => unreachable!(),
    }
}

/// Generates a [`Vec`] of random [`FeatureBlock`]s given a set of allowed [`FeatureBlockFlags`].
pub fn rand_allowed_feature_blocks(allowed_feature_blocks: FeatureBlockFlags) -> Vec<FeatureBlock> {
    let mut all_feature_blocks = FeatureBlockFlags::ALL_FLAGS
        .iter()
        .map(rand_feature_block_from_flag)
        .collect::<Vec<_>>();
    all_feature_blocks.retain(|feature_block| allowed_feature_blocks.contains(feature_block.flag()));
    all_feature_blocks
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    address::rand_address,
    bytes::rand_bytes,
    milestone::rand_milestone_index,
    number::{rand_number, rand_number_range},
};

use bee_message::output::feature_block::{
    DustDepositReturnFeatureBlock, ExpirationFeatureBlock, FeatureBlock, FeatureBlockFlags, IndexationFeatureBlock,
    IssuerFeatureBlock, MetadataFeatureBlock, SenderFeatureBlock, TimelockFeatureBlock,
};

/// Generates a random [`SenderFeatureBlock`].
pub fn rand_sender_feature_block() -> SenderFeatureBlock {
    SenderFeatureBlock::new(rand_address())
}

/// Generates a random [`IssuerFeatureBlock`].
pub fn rand_issuer_feature_block() -> IssuerFeatureBlock {
    IssuerFeatureBlock::new(rand_address())
}

/// Generates a random [`DustDepositReturnFeatureBlock`].
pub fn rand_dust_deposit_return_feature_block() -> DustDepositReturnFeatureBlock {
    DustDepositReturnFeatureBlock::new(rand_number_range(DustDepositReturnFeatureBlock::AMOUNT_RANGE)).unwrap()
}

/// Generates a random [`TimelockFeatureBlock`].
pub fn rand_timelock_feature_block() -> TimelockFeatureBlock {
    TimelockFeatureBlock::new(rand_milestone_index(), rand_number())
}

/// Generates a random [`ExpirationFeatureBlock`].
pub fn rand_expiration_feature_block() -> ExpirationFeatureBlock {
    ExpirationFeatureBlock::new(rand_milestone_index(), rand_number())
}

/// Generates a random [`MetadataFeatureBlock`].
pub fn rand_metadata_feature_block() -> MetadataFeatureBlock {
    let bytes = rand_bytes(rand_number_range(MetadataFeatureBlock::LENGTH_RANGE) as usize);
    MetadataFeatureBlock::new(bytes).unwrap()
}

/// Generates a random [`IndexationFeatureBlock`].
pub fn rand_indexation_feature_block() -> IndexationFeatureBlock {
    let bytes = rand_bytes(rand_number_range(IndexationFeatureBlock::LENGTH_RANGE) as usize);
    IndexationFeatureBlock::new(bytes).unwrap()
}

/// Generates a [`Vec`] of random [`FeatureBlock`]s given a set of allowed [`FeatureBlockFlags`].
pub fn rand_allowed_feature_blocks(allowed_feature_blocks: FeatureBlockFlags) -> Vec<FeatureBlock> {
    let mut all_feature_blocks = FeatureBlockFlags::ALL_FLAGS.to_owned();
    all_feature_blocks.retain(|feature_block| allowed_feature_blocks.contains(feature_block.flag()));
    all_feature_blocks
}

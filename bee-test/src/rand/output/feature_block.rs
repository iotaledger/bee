// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    address::rand_address,
    bytes::rand_bytes,
    milestone::rand_milestone_index,
    number::{rand_number, rand_number_range},
};

use bee_message::{
    constant::{DUST_DEPOSIT_MIN, IOTA_SUPPLY},
    output::feature_block::{
        DustDepositReturnFeatureBlock, ExpirationMilestoneIndexFeatureBlock, ExpirationUnixFeatureBlock, FeatureBlock,
        FeatureBlockFlags, IndexationFeatureBlock, IssuerFeatureBlock, MetadataFeatureBlock, SenderFeatureBlock,
        TimelockMilestoneIndexFeatureBlock, TimelockUnixFeatureBlock,
    },
};

/// Generates a random [`SenderFeatureBlock`].
pub fn rand_sender() -> SenderFeatureBlock {
    SenderFeatureBlock::new(rand_address())
}

/// Generates a random [`IssuerFeatureBlock`].
pub fn rand_issuer() -> IssuerFeatureBlock {
    IssuerFeatureBlock::new(rand_address())
}

/// Generates a random [`DustDepositReturnFeatureBlock`].
pub fn rand_dust_deposit_return() -> DustDepositReturnFeatureBlock {
    DustDepositReturnFeatureBlock::new(rand_number_range(DUST_DEPOSIT_MIN..IOTA_SUPPLY)).unwrap()
}

/// Generates a random [`TimelockMilestoneIndexFeatureBlock`].
pub fn rand_timelock_milestone_index() -> TimelockMilestoneIndexFeatureBlock {
    TimelockMilestoneIndexFeatureBlock::new(rand_milestone_index())
}

/// Generates a random [`TimelockUnixFeatureBlock`].
pub fn rand_timelock_unix() -> TimelockUnixFeatureBlock {
    TimelockUnixFeatureBlock::new(rand_number())
}

/// Generates a random [`ExpirationMilestoneIndexFeatureBlock`].
pub fn rand_expiration_milestone_index() -> ExpirationMilestoneIndexFeatureBlock {
    ExpirationMilestoneIndexFeatureBlock::new(rand_milestone_index())
}

/// Generates a random [`ExpirationUnixFeatureBlock`].
pub fn rand_expiration_unix() -> ExpirationUnixFeatureBlock {
    ExpirationUnixFeatureBlock::new(rand_number())
}

/// Generates a random [`MetadataFeatureBlock`].
pub fn rand_metadata() -> MetadataFeatureBlock {
    let bytes = rand_bytes(rand_number_range(1..MetadataFeatureBlock::LENGTH_MAX));
    MetadataFeatureBlock::new(&bytes).unwrap()
}

/// Generates a random [`IndexationFeatureBlock`].
pub fn rand_indexation() -> IndexationFeatureBlock {
    let bytes = rand_bytes(rand_number_range(1..IndexationFeatureBlock::LENGTH_MAX));
    IndexationFeatureBlock::new(&bytes).unwrap()
}

fn all_feature_blocks() -> Vec<FeatureBlock> {
    vec![
        FeatureBlock::Sender(rand_sender()),
        FeatureBlock::Issuer(rand_issuer()),
        FeatureBlock::DustDepositReturn(rand_dust_deposit_return()),
        FeatureBlock::TimelockMilestoneIndex(rand_timelock_milestone_index()),
        FeatureBlock::TimelockUnix(rand_timelock_unix()),
        FeatureBlock::ExpirationMilestoneIndex(rand_expiration_milestone_index()),
        FeatureBlock::ExpirationUnix(rand_expiration_unix()),
        FeatureBlock::Metadata(rand_metadata()),
        FeatureBlock::Indexation(rand_indexation()),
    ]
}

/// Generates random [`FeatureBlocks`] given a set of allowed [`FeatureBlockFlags`].
pub fn rand_allowed_feature_blocks(allowed_feature_blocks: FeatureBlockFlags) -> Vec<FeatureBlock> {
    let mut all_feature_blocks = all_feature_blocks();
    all_feature_blocks.retain(|feature_block| allowed_feature_blocks.contains(feature_block.flag()));
    all_feature_blocks
}

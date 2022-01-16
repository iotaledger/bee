// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    address::{rand_address, rand_alias_address, rand_ed25519_address},
    bytes::{rand_bytes, rand_bytes_array},
    message::rand_message_id,
    milestone::{rand_milestone_id, rand_milestone_index},
    number::{rand_number, rand_number_range},
    output::feature_block::rand_allowed_feature_blocks,
    transaction::rand_transaction_id,
};

/// Module providing random feature block generation utilities.
pub mod feature_block;

use bee_ledger::types::{ConsumedOutput, CreatedOutput, TreasuryOutput, Unspent};
use bee_message::{
    address::Address,
    output::{self, Output, OutputId, OUTPUT_INDEX_RANGE},
};

use primitive_types::U256;
use rand::prelude::SliceRandom;

/// Generates a random [`OutputId`].
pub fn rand_output_id() -> OutputId {
    OutputId::new(rand_transaction_id(), rand_number_range(OUTPUT_INDEX_RANGE)).unwrap()
}

/// Generates a random [`Unspent`] output id.
pub fn rand_unspent_output_id() -> Unspent {
    Unspent::new(rand_output_id())
}

/// Generates a random treasury output.
pub fn rand_treasury_output() -> output::TreasuryOutput {
    output::TreasuryOutput::new(rand_number_range(output::TreasuryOutput::AMOUNT_RANGE)).unwrap()
}

/// Generates a random ledger [`TreasuryOutput`].
pub fn rand_ledger_treasury_output() -> TreasuryOutput {
    TreasuryOutput::new(rand_treasury_output(), rand_milestone_id())
}

/// Generates a random [`ExtendedOutput`](output::ExtendedOutput).
pub fn rand_extended_output() -> output::ExtendedOutput {
    let feature_blocks = rand_allowed_feature_blocks(output::ExtendedOutput::ALLOWED_FEATURE_BLOCKS);
    // TODO: Add `NativeTokens`
    output::ExtendedOutput::build(rand_address(), rand_number())
        .with_feature_blocks(feature_blocks)
        .finish()
        .unwrap()
}

/// Generates a random [`AliasOutput`](output::AliasOutput).
pub fn rand_alias_output() -> output::AliasOutput {
    let feature_blocks = rand_allowed_feature_blocks(output::AliasOutput::ALLOWED_FEATURE_BLOCKS);

    let ed25519_address = rand_ed25519_address();
    let alias_address = rand_alias_address();

    // We need to make sure that `AliasId` and `Address` don't match.
    let mut alias_id = output::AliasId::from(rand_bytes_array());
    while &alias_id == alias_address.id() {
        alias_id = output::AliasId::from(rand_bytes_array());
    }

    let address_options: [Address; 2] = [ed25519_address.into(), alias_address.into()];

    output::AliasOutput::build(
        rand_number(),
        alias_id,
        *address_options.choose(&mut rand::thread_rng()).unwrap(),
        *address_options.choose(&mut rand::thread_rng()).unwrap(),
    )
    .unwrap()
    .with_feature_blocks(feature_blocks)
    .finish()
    .unwrap()
}

/// Generates a random [`FoundryOutput`](output::FoundryOutput).
pub fn rand_foundry_output() -> output::FoundryOutput {
    let feature_blocks = rand_allowed_feature_blocks(output::FoundryOutput::ALLOWED_FEATURE_BLOCKS);

    let max_supply = U256::from(rand_bytes_array()).saturating_add(1.into());
    let circulating = U256::from(rand_bytes_array()) % max_supply.saturating_add(1.into());

    output::FoundryOutput::build(
        rand_alias_address().into(),
        rand_number(),
        rand_number(),
        rand_bytes_array(),
        circulating,
        max_supply,
        output::TokenScheme::Simple,
    )
    .unwrap()
    .with_feature_blocks(feature_blocks)
    .finish()
    .unwrap()
}

/// Generates a random [`NftOutput`](output::NftOutput).
pub fn rand_nft_output() -> output::NftOutput {
    let feature_blocks = rand_allowed_feature_blocks(output::NftOutput::ALLOWED_FEATURE_BLOCKS);
    output::NftOutput::build(
        rand_address(),
        rand_number(),
        output::NftId::new(rand_bytes_array()),
        rand_bytes(rand_number_range(0..output::NftOutput::IMMUTABLE_METADATA_LENGTH_MAX) as usize),
    )
    .unwrap()
    .with_feature_blocks(feature_blocks)
    .finish()
    .unwrap()
}

/// Generates a random [`Output`].
pub fn rand_output() -> Output {
    match rand_number::<u64>() % 5 {
        0 => rand_treasury_output().into(),
        1 => rand_extended_output().into(),
        2 => rand_alias_output().into(),
        3 => rand_foundry_output().into(),
        4 => rand_nft_output().into(),
        _ => unreachable!(),
    }
}

/// Generates a random [`ConsumedOutput`].
pub fn rand_consumed_output() -> ConsumedOutput {
    ConsumedOutput::new(rand_transaction_id(), rand_milestone_index())
}

/// Generates a random [`CreatedOutput`].
pub fn rand_created_output() -> CreatedOutput {
    CreatedOutput::new(
        rand_message_id(),
        rand_milestone_index(),
        rand_number(),
        match rand_number::<u64>() % 3 {
            // 1 => todo!(),
            _ => rand_treasury_output().into(),
            // _ => unreachable!(),
        },
    )
}

/// Generates a random ledger treasury output.
pub fn rand_ledger_treasury_output() -> TreasuryOutput {
    TreasuryOutput::new(rand_treasury_output(), rand_milestone_id())
}

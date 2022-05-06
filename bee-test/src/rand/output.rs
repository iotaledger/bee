// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Module providing random feature block generation utilities.
pub mod feature_block;
/// Module providing random unlock condition generation utilities.
pub mod unlock_condition;

use bee_ledger::types::{ConsumedOutput, CreatedOutput, TreasuryOutput, Unspent};
use bee_message::output::{
    self, unlock_condition::ImmutableAliasAddressUnlockCondition, InputsCommitment, Output, OutputId,
    SimpleTokenScheme, TokenScheme, OUTPUT_INDEX_RANGE,
};
use primitive_types::U256;

use crate::rand::{
    address::rand_alias_address,
    bytes::rand_bytes_array,
    message::rand_message_id,
    milestone::{rand_milestone_id, rand_milestone_index},
    number::{rand_number, rand_number_range},
    output::{
        feature_block::rand_allowed_feature_blocks,
        unlock_condition::{
            rand_address_unlock_condition, rand_address_unlock_condition_different_from,
            rand_governor_address_unlock_condition_different_from,
            rand_state_controller_address_unlock_condition_different_from,
        },
    },
    transaction::rand_transaction_id,
};

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

/// Generates a random [`BasicOutput`](output::BasicOutput).
pub fn rand_basic_output() -> output::BasicOutput {
    let feature_blocks = rand_allowed_feature_blocks(output::BasicOutput::ALLOWED_FEATURE_BLOCKS);
    // TODO: Add `NativeTokens`
    output::BasicOutput::build_with_amount(rand_number_range(Output::AMOUNT_RANGE))
        .unwrap()
        .with_feature_blocks(feature_blocks)
        .add_unlock_condition(rand_address_unlock_condition().into())
        .finish()
        .unwrap()
}

/// Generates a random [`AliasOutput`](output::AliasOutput).
pub fn rand_alias_output() -> output::AliasOutput {
    let feature_blocks = rand_allowed_feature_blocks(output::AliasOutput::ALLOWED_FEATURE_BLOCKS);

    // We need to make sure that `AliasId` and `Address` don't match.
    let alias_id = output::AliasId::from(rand_bytes_array());

    output::AliasOutput::build_with_amount(rand_number_range(Output::AMOUNT_RANGE), alias_id)
        .unwrap()
        .with_feature_blocks(feature_blocks)
        .add_unlock_condition(rand_state_controller_address_unlock_condition_different_from(&alias_id).into())
        .add_unlock_condition(rand_governor_address_unlock_condition_different_from(&alias_id).into())
        .finish()
        .unwrap()
}

/// Generates a random [`TokenScheme`].
pub fn rand_token_scheme() -> TokenScheme {
    let max = U256::from(rand_bytes_array()).saturating_add(U256::one());
    let minted = U256::from(rand_bytes_array()) % max.saturating_add(U256::one());
    let melted = U256::from(rand_bytes_array()) % minted.saturating_add(U256::one());

    TokenScheme::Simple(SimpleTokenScheme::new(minted, melted, max).unwrap())
}

/// Generates a random [`FoundryOutput`](output::FoundryOutput).
pub fn rand_foundry_output() -> output::FoundryOutput {
    let feature_blocks = rand_allowed_feature_blocks(output::FoundryOutput::ALLOWED_FEATURE_BLOCKS);

    output::FoundryOutput::build_with_amount(
        rand_number_range(Output::AMOUNT_RANGE),
        rand_number(),
        rand_token_scheme(),
    )
    .unwrap()
    .with_feature_blocks(feature_blocks)
    .add_unlock_condition(ImmutableAliasAddressUnlockCondition::new(rand_alias_address()).into())
    .finish()
    .unwrap()
}

/// Generates a random [`NftOutput`](output::NftOutput).
pub fn rand_nft_output() -> output::NftOutput {
    let feature_blocks = rand_allowed_feature_blocks(output::NftOutput::ALLOWED_FEATURE_BLOCKS);

    // We need to make sure that `NftId` and `Address` don't match.
    let nft_id = output::NftId::from(rand_bytes_array());

    output::NftOutput::build_with_amount(rand_number_range(Output::AMOUNT_RANGE), nft_id)
        .unwrap()
        .with_feature_blocks(feature_blocks)
        .add_unlock_condition(rand_address_unlock_condition_different_from(&nft_id).into())
        .finish()
        .unwrap()
}

/// Generates a random ledger [`TreasuryOutput`].
pub fn rand_ledger_treasury_output() -> TreasuryOutput {
    TreasuryOutput::new(rand_treasury_output(), rand_milestone_id())
}

/// Generates a random [`Output`].
pub fn rand_output() -> Output {
    match rand_number::<u64>() % 5 {
        0 => rand_treasury_output().into(),
        1 => rand_basic_output().into(),
        2 => rand_alias_output().into(),
        3 => rand_foundry_output().into(),
        4 => rand_nft_output().into(),
        _ => unreachable!(),
    }
}

/// Generates a random [`ConsumedOutput`].
pub fn rand_consumed_output() -> ConsumedOutput {
    ConsumedOutput::new(rand_transaction_id(), rand_milestone_index(), rand_number())
}

/// Generates a random [`CreatedOutput`].
pub fn rand_created_output() -> CreatedOutput {
    CreatedOutput::new(rand_message_id(), rand_milestone_index(), rand_number(), rand_output())
}

/// Generates a random [`InputsCommitment`].
pub fn rand_inputs_commitment() -> InputsCommitment {
    InputsCommitment::from(rand_bytes_array())
}

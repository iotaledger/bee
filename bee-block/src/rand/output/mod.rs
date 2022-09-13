// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Module providing random feature generation utilities.
pub mod feature;
/// Module providing random unlock condition generation utilities.
pub mod unlock_condition;

use primitive_types::U256;

use crate::{
    output::{
        unlock_condition::ImmutableAliasAddressUnlockCondition, AliasId, AliasOutput, BasicOutput, FoundryOutput,
        InputsCommitment, NftId, NftOutput, Output, OutputId, SimpleTokenScheme, TokenScheme, TreasuryOutput,
        OUTPUT_INDEX_RANGE,
    },
    protocol::ProtocolParameters,
    rand::{
        address::rand_alias_address,
        bytes::rand_bytes_array,
        number::{rand_number, rand_number_range},
        output::{
            feature::rand_allowed_features,
            unlock_condition::{
                rand_address_unlock_condition, rand_address_unlock_condition_different_from,
                rand_governor_address_unlock_condition_different_from,
                rand_state_controller_address_unlock_condition_different_from,
            },
        },
        transaction::rand_transaction_id,
    },
};

/// Generates a random [`OutputId`].
pub fn rand_output_id() -> OutputId {
    OutputId::new(rand_transaction_id(), rand_number_range(OUTPUT_INDEX_RANGE)).unwrap()
}

/// Generates a random treasury output.
pub fn rand_treasury_output(protocol_parameters: &ProtocolParameters) -> TreasuryOutput {
    TreasuryOutput::new(
        rand_number_range(0..protocol_parameters.token_supply()),
        protocol_parameters,
    )
    .unwrap()
}

/// Generates a random [`BasicOutput`](BasicOutput).
pub fn rand_basic_output(protocol_parameters: &ProtocolParameters) -> BasicOutput {
    let features = rand_allowed_features(BasicOutput::ALLOWED_FEATURES);
    // TODO: Add `NativeTokens`
    BasicOutput::build_with_amount(rand_number_range(
        Output::AMOUNT_MIN..protocol_parameters.token_supply(),
    ))
    .unwrap()
    .with_features(features)
    .add_unlock_condition(rand_address_unlock_condition().into())
    .finish(protocol_parameters)
    .unwrap()
}

/// Generates a random [`AliasId`](AliasId).
pub fn rand_alias_id() -> AliasId {
    AliasId::from(rand_bytes_array())
}

/// Generates a random [`AliasOutput`](AliasOutput).
pub fn rand_alias_output(protocol_parameters: &ProtocolParameters) -> AliasOutput {
    let features = rand_allowed_features(AliasOutput::ALLOWED_FEATURES);

    // We need to make sure that `AliasId` and `Address` don't match.
    let alias_id = rand_alias_id();

    AliasOutput::build_with_amount(
        rand_number_range(Output::AMOUNT_MIN..protocol_parameters.token_supply()),
        alias_id,
    )
    .unwrap()
    .with_features(features)
    .add_unlock_condition(rand_state_controller_address_unlock_condition_different_from(&alias_id).into())
    .add_unlock_condition(rand_governor_address_unlock_condition_different_from(&alias_id).into())
    .finish(protocol_parameters)
    .unwrap()
}

/// Generates a random [`TokenScheme`].
pub fn rand_token_scheme() -> TokenScheme {
    let max = U256::from(rand_bytes_array()).saturating_add(U256::one());
    let minted = U256::from(rand_bytes_array()) % max.saturating_add(U256::one());
    let melted = U256::from(rand_bytes_array()) % minted.saturating_add(U256::one());

    TokenScheme::Simple(SimpleTokenScheme::new(minted, melted, max).unwrap())
}

/// Generates a random [`FoundryOutput`](FoundryOutput).
pub fn rand_foundry_output(protocol_parameters: &ProtocolParameters) -> FoundryOutput {
    let features = rand_allowed_features(FoundryOutput::ALLOWED_FEATURES);

    FoundryOutput::build_with_amount(
        rand_number_range(Output::AMOUNT_MIN..protocol_parameters.token_supply()),
        rand_number(),
        rand_token_scheme(),
    )
    .unwrap()
    .with_features(features)
    .add_unlock_condition(ImmutableAliasAddressUnlockCondition::new(rand_alias_address()).into())
    .finish(protocol_parameters)
    .unwrap()
}

/// Generates a random [`NftOutput`](NftOutput).
pub fn rand_nft_output(protocol_parameters: &ProtocolParameters) -> NftOutput {
    let features = rand_allowed_features(NftOutput::ALLOWED_FEATURES);

    // We need to make sure that `NftId` and `Address` don't match.
    let nft_id = NftId::from(rand_bytes_array());

    NftOutput::build_with_amount(
        rand_number_range(Output::AMOUNT_MIN..protocol_parameters.token_supply()),
        nft_id,
    )
    .unwrap()
    .with_features(features)
    .add_unlock_condition(rand_address_unlock_condition_different_from(&nft_id).into())
    .finish(protocol_parameters)
    .unwrap()
}

/// Generates a random [`InputsCommitment`].
pub fn rand_inputs_commitment() -> InputsCommitment {
    InputsCommitment::from(rand_bytes_array())
}

/// Generates a random [`Output`].
pub fn rand_output(protocol_parameters: &ProtocolParameters) -> Output {
    match rand_number::<u64>() % 5 {
        0 => rand_treasury_output(protocol_parameters).into(),
        1 => rand_basic_output(protocol_parameters).into(),
        2 => rand_alias_output(protocol_parameters).into(),
        3 => rand_foundry_output(protocol_parameters).into(),
        4 => rand_nft_output(protocol_parameters).into(),
        _ => unreachable!(),
    }
}

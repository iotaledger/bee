// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::output::{ByteCost, ByteCostConfig, ByteCostConfigBuilder, Output};
use bee_test::rand::output::{rand_alias_output, rand_basic_output, rand_foundry_output, rand_nft_output};

const BYTE_COST: u64 = 1;
const FACTOR_KEY: u64 = 10;
const FACTOR_DATA: u64 = 1;

fn config() -> ByteCostConfig {
    ByteCostConfigBuilder::new()
        .byte_cost(BYTE_COST)
        .key_factor(FACTOR_KEY)
        .data_factor(FACTOR_DATA)
        .finish()
}

fn output_in_range(output: Output, range: std::ops::RangeInclusive<u64>) {
    let cost = output.byte_cost(&config());
    assert!(
        range.contains(&cost),
        "{:#?} has a required byte cost of {}",
        output,
        cost
    );
}

#[test]
fn valid_byte_cost_range() {
    output_in_range(Output::Alias(rand_alias_output()), 445..=29_620);
    output_in_range(Output::Basic(rand_basic_output()), 414..=13_485);
    output_in_range(Output::Foundry(rand_foundry_output()), 496..=21_365);
    output_in_range(Output::Nft(rand_nft_output()), 435..=21_734);
}

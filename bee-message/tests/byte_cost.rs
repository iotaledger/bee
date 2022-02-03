// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::output::Output;
use bee_test::rand::output::{rand_alias_output, rand_extended_output, rand_foundry_output, rand_nft_output};

use bee_byte_cost::{ByteCost, ByteCostConfig};

const CONFIG: ByteCostConfig = ByteCostConfig {
    byte_cost: 1,
    weight_for_data: 10,
    weight_for_key: 1,
};

const OFFSET: u64 = 34 /* OutputID */
                  + 32 /* Message ID */
                  + 4  /* Confirmation Milestone Index */
                  + 4  /* Confirmation Unix Timestamp */;

fn output_in_range(output: Output, range: std::ops::RangeInclusive<u64>) {
    let v_bytes = &output.weighted_bytes(&CONFIG);
    assert!(range.contains(v_bytes), "{:#?} has byte cost {}", output, v_bytes);
}

#[test]
fn valid_byte_cost_range() {
    output_in_range(Output::Alias(rand_alias_output()), (444 - OFFSET)..=(17_505 - OFFSET));
    output_in_range(
        Output::Extended(rand_extended_output()),
        (414 - OFFSET)..=(9_565 - OFFSET),
    );
    output_in_range(
        Output::Foundry(rand_foundry_output()),
        (495 - OFFSET)..=(9_250 - OFFSET),
    );
    output_in_range(Output::Nft(rand_nft_output()), (436 - OFFSET)..=(17_813 - OFFSET));
}

// TODO: Add tests from Hornet too.

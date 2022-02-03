// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_byte_cost::{ByteCost, ByteCostConfig};

const CONFIG: ByteCostConfig = ByteCostConfig {
    byte_cost: 1,
    weight_for_data: 10,
    weight_for_key: 1,
};

#[cfg(test)]
mod output {
    use super::*;

    use bee_test::rand::{output::{rand_alias_output}, bytes::rand_bytes_array, transaction::rand_transaction_id};

    #[test]
    fn alias_ouput_in_range() {
        let output = rand_alias_output();
        let byte_cost_range = 444..=17_505;
        assert!(byte_cost_range.contains(&output.weighted_bytes(&CONFIG)));
    }
}

// TODO: Add tests from Hornet too.

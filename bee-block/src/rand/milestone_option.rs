// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::milestone::ReceiptMilestoneOption,
    rand::{
        bool::rand_bool, milestone::rand_milestone_index, payload::rand_treasury_transaction_payload,
        receipt::rand_migrated_funds_entry,
    },
};

/// Generates a random receipt milestone option.
pub fn rand_receipt_milestone_option(token_supply: u64) -> ReceiptMilestoneOption {
    ReceiptMilestoneOption::new(
        rand_milestone_index(),
        rand_bool(),
        vec![rand_migrated_funds_entry(token_supply)],
        rand_treasury_transaction_payload(token_supply),
        token_supply,
    )
    .unwrap()
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::milestone::ReceiptMilestoneOption,
    protocol::ProtocolParameters,
    rand::{
        bool::rand_bool, milestone::rand_milestone_index, payload::rand_treasury_transaction_payload,
        receipt::rand_migrated_funds_entry,
    },
};

/// Generates a random receipt milestone option.
pub fn rand_receipt_milestone_option(protocol_parameters: &ProtocolParameters) -> ReceiptMilestoneOption {
    ReceiptMilestoneOption::new(
        rand_milestone_index(),
        rand_bool(),
        vec![rand_migrated_funds_entry(protocol_parameters)],
        rand_treasury_transaction_payload(protocol_parameters.token_supply()),
        protocol_parameters.token_supply(),
    )
    .unwrap()
}

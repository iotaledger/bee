// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    address::rand_address,
    bool::rand_bool,
    bytes::rand_bytes,
    integer::{rand_integer, rand_integer_range},
    message::rand_message_id,
    milestone::rand_milestone_index,
};

use bee_ledger::model::Receipt;
use bee_message::{
    input::{Input, TreasuryInput},
    output::{Output, SignatureLockedSingleOutput, TreasuryOutput, TREASURY_OUTPUT_AMOUNT},
    payload::{
        receipt::{MigratedFundsEntry, ReceiptPayload, MIGRATED_FUNDS_ENTRY_AMOUNT},
        treasury::TreasuryTransactionPayload,
        Payload,
    },
};

use std::convert::TryInto;

// TODO move
pub fn rand_treasury_input() -> Input {
    TreasuryInput::new(rand_message_id()).into()
}

// TODO move
pub fn rand_treasury_output() -> Output {
    TreasuryOutput::new(rand_integer_range(TREASURY_OUTPUT_AMOUNT))
        .unwrap()
        .into()
}

// TODO move
pub fn rand_treasury_transaction() -> Payload {
    TreasuryTransactionPayload::new(rand_treasury_input(), rand_treasury_output())
        .unwrap()
        .into()
}

pub fn rand_migrated_funds_entry() -> MigratedFundsEntry {
    MigratedFundsEntry::new(
        rand_bytes(49).try_into().unwrap(),
        SignatureLockedSingleOutput::new(rand_address(), rand_integer_range(MIGRATED_FUNDS_ENTRY_AMOUNT)).unwrap(),
    )
    .unwrap()
}

pub fn rand_receipt() -> Receipt {
    // TODO rand vector
    Receipt::new(
        ReceiptPayload::new(
            rand_integer(),
            rand_bool(),
            vec![rand_migrated_funds_entry()],
            rand_treasury_transaction(),
        )
        .unwrap(),
        rand_milestone_index(),
    )
}

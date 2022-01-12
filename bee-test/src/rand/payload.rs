// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bool::rand_bool, bytes::rand_bytes, input::rand_treasury_input, milestone::rand_milestone_index,
    number::rand_number_range, output::rand_treasury_output, receipt::rand_migrated_funds_entry,
};

use bee_message::payload::{
    indexation::IndexationPayload, receipt::ReceiptPayload, treasury::TreasuryTransactionPayload, Payload,
};

/// Generates a random indexation payload.
pub fn rand_indexation_payload() -> IndexationPayload {
    IndexationPayload::new(
        rand_bytes(rand_number_range(IndexationPayload::LENGTH_RANGE).into()),
        rand_bytes(rand_number_range(0..10000)),
    )
    .unwrap()
}

/// Generates a random treasury transaction payload.
pub fn rand_treasury_transaction_payload() -> TreasuryTransactionPayload {
    TreasuryTransactionPayload::new(rand_treasury_input().into(), rand_treasury_output().into()).unwrap()
}

/// Generates a random receipt payload.
pub fn rand_receipt_payload() -> ReceiptPayload {
    ReceiptPayload::new(
        rand_milestone_index(),
        rand_bool(),
        vec![rand_migrated_funds_entry()],
        rand_treasury_transaction_payload().into(),
    )
    .unwrap()
}

/// Generates a random payload for a message.
pub fn rand_payload_for_message() -> Payload {
    // TODO complete
    rand_indexation_payload().into()
}

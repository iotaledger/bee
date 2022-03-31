// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::{
    receipt::ReceiptPayload, tagged_data::TaggedDataPayload, treasury_transaction::TreasuryTransactionPayload, Payload,
};

use crate::rand::{
    bool::rand_bool, bytes::rand_bytes, input::rand_treasury_input, milestone::rand_milestone_index,
    number::rand_number_range, output::rand_treasury_output, receipt::rand_migrated_funds_entry,
};

/// Generates a random tagged data payload.
pub fn rand_tagged_data_payload() -> TaggedDataPayload {
    TaggedDataPayload::new(
        rand_bytes(rand_number_range(TaggedDataPayload::TAG_LENGTH_RANGE).into()),
        rand_bytes(rand_number_range(0..10000)),
    )
    .unwrap()
}

/// Generates a random treasury transaction payload.
pub fn rand_treasury_transaction_payload() -> TreasuryTransactionPayload {
    TreasuryTransactionPayload::new(rand_treasury_input(), rand_treasury_output()).unwrap()
}

/// Generates a random receipt payload.
pub fn rand_receipt_payload() -> ReceiptPayload {
    ReceiptPayload::new(
        rand_milestone_index(),
        rand_bool(),
        vec![rand_migrated_funds_entry()],
        rand_treasury_transaction_payload(),
    )
    .unwrap()
}

/// Generates a random payload for a message.
pub fn rand_payload_for_message() -> Payload {
    // TODO complete
    rand_tagged_data_payload().into()
}

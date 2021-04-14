// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bytes::{rand_bytes, rand_bytes_32},
    input::rand_treasury_input,
    output::rand_treasury_output,
};

use bee_message::payload::{indexation::IndexationPayload, treasury::TreasuryTransactionPayload, Payload};

/// Generates a random treasury transaction.
pub fn rand_treasury_transaction() -> Payload {
    TreasuryTransactionPayload::new(rand_treasury_input(), rand_treasury_output())
        .unwrap()
        .into()
}

/// Generates a random indexation payload.
pub fn rand_indexation() -> IndexationPayload {
    IndexationPayload::new(&rand_bytes_32(), &rand_bytes(64)).unwrap()
}

/// Generates a random payload.
pub fn rand_payload() -> Payload {
    rand_indexation().into()
}

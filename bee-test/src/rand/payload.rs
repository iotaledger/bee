// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    payload::{
        milestone::{MilestoneEssence, MilestoneOptions, MilestonePayload},
        tagged_data::TaggedDataPayload,
        treasury_transaction::TreasuryTransactionPayload,
        Payload,
    },
    signature::{Ed25519Signature, Signature},
};

use crate::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    input::rand_treasury_input,
    milestone::{rand_milestone_id, rand_milestone_index},
    number::{rand_number, rand_number_range},
    output::rand_treasury_output,
    parents::rand_parents,
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

/// Generates a random milestone payload.
pub fn rand_milestone_payload() -> MilestonePayload {
    let essence = MilestoneEssence::new(
        rand_milestone_index(),
        rand_number(),
        rand_milestone_id(),
        rand_parents(),
        rand_bytes_array(),
        rand_bytes_array(),
        rand_bytes(32),
        MilestoneOptions::new(vec![]).unwrap(),
    )
    .unwrap();
    let signatures = vec![Signature::from(Ed25519Signature::new(
        rand_bytes_array(),
        rand_bytes_array(),
    ))];

    MilestonePayload::new(essence, signatures).unwrap()
}

/// Generates a random payload for a message.
pub fn rand_payload_for_message() -> Payload {
    // TODO complete
    rand_tagged_data_payload().into()
}

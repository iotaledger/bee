// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bool::rand_bool,
    bytes::rand_bytes_array,
    message::{input::rand_input, output::rand_outputs, payload::rand_indexation_payload, unlock::rand_unlocks},
    number::{rand_number, rand_number_range},
    vec::rand_vec,
};

use bee_message::{
    payload::transaction::{TransactionEssence, TransactionId, TransactionPayload},
    unlock::UnlockBlocks,
};
use bee_packable::Packable;

/// Generates a random [`TransactionId`].
pub fn rand_transaction_id() -> TransactionId {
    TransactionId::new(rand_bytes_array())
}

/// Generates a random [`TransactionPayload`].
pub fn rand_transaction_payload() -> TransactionPayload {
    let mut inputs = rand_vec(rand_input, rand_number_range(1..=127));
    inputs.sort_by(|a, b| Packable::pack_to_vec(a).partial_cmp(&Packable::pack_to_vec(b)).unwrap());

    let mut outputs = rand_outputs(rand_number_range(1..127));
    outputs.sort_by(|a, b| Packable::pack_to_vec(a).partial_cmp(&Packable::pack_to_vec(b)).unwrap());

    let unlock_blocks = rand_unlocks(inputs.len());

    let mut essence_builder = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(inputs)
        .with_outputs(outputs);

    if rand_bool() {
        essence_builder = essence_builder.with_payload(rand_indexation_payload().into());
    }

    TransactionPayload::builder()
        .with_essence(essence_builder.finish().unwrap())
        .with_unlock_blocks(UnlockBlocks::new(unlock_blocks).unwrap())
        .finish()
        .unwrap()
}

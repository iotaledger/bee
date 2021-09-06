// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bytes::rand_bytes_array,
    message::{input::rand_input, output::rand_outputs, unlock::rand_unlocks},
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
    inputs.sort_by(|a, b| {
        Packable::pack_to_vec(a)
            .unwrap()
            .partial_cmp(&Packable::pack_to_vec(b).unwrap())
            .unwrap()
    });

    let mut outputs = rand_outputs(rand_number_range(1..127));
    outputs.sort_by(|a, b| {
        Packable::pack_to_vec(a)
            .unwrap()
            .partial_cmp(&Packable::pack_to_vec(b).unwrap())
            .unwrap()
    });

    let unlock_blocks = rand_unlocks(inputs.len());

    TransactionPayload::builder()
        .with_essence(
            TransactionEssence::builder()
                .with_timestamp(rand_number())
                .with_access_pledge_id(rand_bytes_array())
                .with_consensus_pledge_id(rand_bytes_array())
                .with_inputs(inputs)
                .with_outputs(outputs)
                .finish()
                .unwrap(),
        )
        .with_unlock_blocks(UnlockBlocks::new(unlock_blocks).unwrap())
        .finish()
        .unwrap()
}

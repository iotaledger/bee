// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bytes::rand_bytes_array, number::rand_number, parents::rand_parents, payload::rand_payload_for_message,
};

use bee_message::{parent::Parents, Message, MessageBuilder, MessageId};

/// Generates a random message id.
pub fn rand_message_id() -> MessageId {
    MessageId::new(rand_bytes_array())
}

/// Generates a vector of random message ids of a given length.
pub fn rand_message_ids(len: usize) -> Vec<MessageId> {
    let mut parents = (0..len).map(|_| rand_message_id()).collect::<Vec<_>>();
    parents.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
    parents
}

/// Generates a random message with given parents.
pub fn rand_message_with_parents(parents: Parents) -> Message {
    MessageBuilder::<u64>::new()
        .with_network_id(rand_number())
        .with_parents(parents)
        .with_payload(rand_payload_for_message())
        .with_nonce_provider(rand_number(), 0f64)
        .finish()
        .unwrap()
}

/// Generates a random message.
pub fn rand_message() -> Message {
    rand_message_with_parents(rand_parents())
}

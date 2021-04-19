// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bytes::{rand_bytes, rand_bytes_32},
    integer::rand_integer,
    parents::rand_parents,
};

use bee_message::{
    parents::Parents,
    payload::{indexation::IndexationPayload, Payload},
    Message, MessageBuilder, MessageId,
};

pub fn rand_message_id() -> MessageId {
    MessageId::new(rand_bytes_32())
}

pub fn rand_message_ids(len: usize) -> Vec<MessageId> {
    let mut parents = (0..len).map(|_| rand_message_id()).collect::<Vec<MessageId>>();
    parents.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
    parents
}

pub fn rand_indexation() -> IndexationPayload {
    IndexationPayload::new(&rand_bytes_32(), &rand_bytes(64)).unwrap()
}

pub fn rand_payload() -> Payload {
    // TODO complete with other types
    rand_indexation().into()
}

pub fn rand_message_with_parents(parents: Parents) -> Message {
    MessageBuilder::<u64>::new()
        .with_network_id(rand_integer())
        .with_parents(parents)
        .with_payload(rand_payload())
        .with_nonce_provider(rand_integer(), 0f64)
        .finish()
        .unwrap()
}

pub fn rand_message() -> Message {
    rand_message_with_parents(rand_parents())
}

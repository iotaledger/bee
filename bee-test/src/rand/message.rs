// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bytes::{random_bytes, random_bytes_32},
    integer::random_integer,
    string::random_string,
};

use bee_message::{
    payload::{indexation::Indexation, Payload},
    Message, MessageId,
};

pub fn random_message_id() -> MessageId {
    MessageId::new(random_bytes_32())
}

pub fn random_indexation() -> Indexation {
    Indexation::new(random_string(32), &random_bytes(64)).unwrap()
}

pub fn random_payload() -> Payload {
    // TODO complete with other types
    random_indexation().into()
}

pub fn random_message_with_parents(parent1: MessageId, parent2: MessageId) -> Message {
    Message::builder()
        .with_network_id(random_integer())
        .with_parent1(parent1)
        .with_parent2(parent2)
        .with_payload(random_payload())
        .with_nonce(random_integer())
        .finish()
        .unwrap()
}

pub fn random_message() -> Message {
    random_message_with_parents(random_message_id(), random_message_id())
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bytes::{rand_bytes, rand_bytes_32},
    integer::rand_integer,
    string::rand_string,
};

use bee_message::{
    payload::{indexation::IndexationPayload, Payload},
    Message, MessageBuilder, MessageId,
};
use bee_pow::providers::{Constant, ConstantBuilder, ProviderBuilder};

pub fn rand_message_id() -> MessageId {
    MessageId::new(rand_bytes_32())
}

pub fn rand_indexation() -> IndexationPayload {
    IndexationPayload::new(rand_string(32), &rand_bytes(64)).unwrap()
}

pub fn rand_payload() -> Payload {
    // TODO complete with other types
    rand_indexation().into()
}

pub fn rand_message_with_parents(parent1: MessageId, parent2: MessageId) -> Message {
    MessageBuilder::<Constant>::new()
        .with_network_id(rand_integer())
        .with_parent1(parent1)
        .with_parent2(parent2)
        .with_payload(rand_payload())
        .with_nonce_provider(ConstantBuilder::new().with_value(rand_integer()).finish(), 0f64)
        .finish()
        .unwrap()
}

pub fn rand_message() -> Message {
    rand_message_with_parents(rand_message_id(), rand_message_id())
}

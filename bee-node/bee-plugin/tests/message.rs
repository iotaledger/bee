// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    payload::{indexation::IndexationPayload, Payload},
    Message, MessageBuilder,
};
use bee_test::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    message::parents::rand_parents,
    number::rand_number,
};

#[test]
fn round_trip_conversion() {
    let message = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(Payload::from(
            IndexationPayload::new(rand_bytes(32), rand_bytes(256)).unwrap(),
        ))
        .with_nonce(0)
        .with_signature(rand_bytes_array())
        .finish()
        .unwrap();

    let protobuf_message = bee_plugin::message::Message::from(&message);
    let new_message: Message = protobuf_message.into();
    assert_eq!(message, new_message);
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    parents::{ParentsBlock, ParentsKind},
    payload::{indexation::IndexationPayload, Payload},
    util::hex_decode,
    Message, MessageBuilder, MessageId,
};
use bee_test::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    number::rand_number,
};

const PARENT_1: &str = "4cfd028fe4789dd3f4518cb67810c77772c0af52261fc767e68b64015931849e";
const PARENT_2: &str = "9bbda9ed78333088a81c73842e242a34e56703c389cba974b11d83828f421a82";
const PARENT_3: &str = "c3186d4e99c8e10b9529e56a54e6d7052c74b84221394c825f452eba633f2c9f";

#[test]
fn round_trip_conversion() {
    let message = MessageBuilder::new()
        .with_parents_blocks(vec![
            ParentsBlock::new(ParentsKind::Strong, vec![MessageId::new(hex_decode(PARENT_1).unwrap())]).unwrap(),
            ParentsBlock::new(
                ParentsKind::Weak,
                vec![
                    MessageId::new(hex_decode(PARENT_2).unwrap()),
                    MessageId::new(hex_decode(PARENT_3).unwrap()),
                ],
            )
            .unwrap(),
        ])
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

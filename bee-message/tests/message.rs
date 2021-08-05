// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    parents::{ParentsBlock, ParentsKind},
    payload::{indexation::IndexationPayload, Payload},
    util::hex_decode,
    Message, MessageBuilder, MessageId, MessageUnpackError, ValidationError,
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    number::rand_number,
};

const PARENT_1: &str = "4cfd028fe4789dd3f4518cb67810c77772c0af52261fc767e68b64015931849e";
const PARENT_2: &str = "9bbda9ed78333088a81c73842e242a34e56703c389cba974b11d83828f421a82";
const PARENT_3: &str = "c3186d4e99c8e10b9529e56a54e6d7052c74b84221394c825f452eba633f2c9f";
const PARENT_4: &str = "da625de8219fab8672ff28c1fa1105772dd5e49107e37172a42d564535254008";
const PARENT_5: &str = "f1109e0f2f5532ba8e3ee6838c68d019b4bbf0b5eeb947ff0a7a8154cfb81cc0";

#[test]
fn new_valid() {
    let message = MessageBuilder::new()
        .add_parents_block(
            ParentsBlock::new(ParentsKind::Strong, vec![MessageId::new(hex_decode(PARENT_1).unwrap())]).unwrap(),
        )
        .add_parents_block(
            ParentsBlock::new(
                ParentsKind::Weak,
                vec![
                    MessageId::new(hex_decode(PARENT_2).unwrap()),
                    MessageId::new(hex_decode(PARENT_3).unwrap()),
                ],
            )
            .unwrap(),
        )
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(Payload::from(
            IndexationPayload::new(rand_bytes(32), rand_bytes(256)).unwrap(),
        ))
        .with_nonce(0)
        .with_signature(rand_bytes_array())
        .finish();

    assert!(message.is_ok());
}

#[test]
fn unpack_valid() {
    let bytes = vec![
        1, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
        2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 238, 142, 220, 195, 72, 99,
        77, 135, 73, 71, 196, 160, 101, 213, 130, 203, 214, 96, 245, 30, 3, 44, 37, 103, 128, 55, 240, 155, 139, 220,
        142, 178, 216, 230, 192, 191, 209, 104, 112, 20, 2, 0, 0, 0, 1, 104, 0, 0, 0, 8, 0, 0, 0, 0, 32, 0, 0, 0, 132,
        27, 114, 220, 115, 116, 126, 193, 10, 134, 212, 173, 149, 101, 177, 183, 239, 215, 196, 68, 91, 60, 110, 222,
        214, 229, 233, 139, 78, 192, 242, 72, 64, 0, 0, 0, 153, 128, 64, 149, 20, 34, 176, 142, 218, 58, 195, 204, 46,
        40, 206, 2, 5, 166, 147, 196, 253, 226, 199, 30, 119, 83, 20, 169, 249, 80, 123, 20, 163, 123, 208, 238, 69,
        191, 198, 110, 105, 107, 184, 244, 12, 51, 64, 199, 121, 8, 14, 248, 38, 118, 144, 2, 133, 4, 126, 169, 122,
        117, 124, 134, 0, 0, 0, 0, 0, 0, 0, 0, 145, 167, 69, 239, 139, 44, 177, 36, 175, 85, 127, 123, 121, 5, 53, 252,
        47, 72, 99, 133, 46, 48, 76, 67, 166, 136, 216, 171, 49, 120, 150, 197, 94, 234, 36, 251, 59, 102, 43, 196, 54,
        55, 138, 254, 248, 226, 27, 75, 64, 65, 70, 179, 143, 249, 27, 85, 91, 169, 46, 237, 98, 213, 205, 27,
    ];

    let message = Message::unpack_from_slice(bytes);

    assert!(message.is_ok());
}

#[test]
fn unpack_invalid_version() {
    let bytes = vec![
        0, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
        2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 238, 142, 220, 195, 72, 99,
        77, 135, 73, 71, 196, 160, 101, 213, 130, 203, 214, 96, 245, 30, 3, 44, 37, 103, 128, 55, 240, 155, 139, 220,
        142, 178, 216, 230, 192, 191, 209, 104, 112, 20, 2, 0, 0, 0, 1, 8, 0, 0, 0, 0, 32, 0, 0, 0, 132, 27, 114, 220,
        115, 116, 126, 193, 10, 134, 212, 173, 149, 101, 177, 183, 239, 215, 196, 68, 91, 60, 110, 222, 214, 229, 233,
        139, 78, 192, 242, 72, 64, 0, 0, 0, 153, 128, 64, 149, 20, 34, 176, 142, 218, 58, 195, 204, 46, 40, 206, 2, 5,
        166, 147, 196, 253, 226, 199, 30, 119, 83, 20, 169, 249, 80, 123, 20, 163, 123, 208, 238, 69, 191, 198, 110,
        105, 107, 184, 244, 12, 51, 64, 199, 121, 8, 14, 248, 38, 118, 144, 2, 133, 4, 126, 169, 122, 117, 124, 134, 0,
        0, 0, 0, 0, 0, 0, 0, 145, 167, 69, 239, 139, 44, 177, 36, 175, 85, 127, 123, 121, 5, 53, 252, 47, 72, 99, 133,
        46, 48, 76, 67, 166, 136, 216, 171, 49, 120, 150, 197, 94, 234, 36, 251, 59, 102, 43, 196, 54, 55, 138, 254,
        248, 226, 27, 75, 64, 65, 70, 179, 143, 249, 27, 85, 91, 169, 46, 237, 98, 213, 205, 27,
    ];

    let message = Message::unpack_from_slice(bytes);

    assert!(matches!(
        message.err().unwrap(),
        UnpackError::Packable(MessageUnpackError::Validation(
            ValidationError::InvalidMessageVersion(0)
        ))
    ));
}

#[test]
fn invalid_no_strong_parents() {
    let message = MessageBuilder::new()
        .add_parents_block(
            ParentsBlock::new(
                ParentsKind::Weak,
                vec![
                    MessageId::new(hex_decode(PARENT_1).unwrap()),
                    MessageId::new(hex_decode(PARENT_2).unwrap()),
                ],
            )
            .unwrap(),
        )
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(Payload::from(
            IndexationPayload::new(rand_bytes(32), rand_bytes(256)).unwrap(),
        ))
        .with_nonce(0)
        .with_signature(rand_bytes_array())
        .finish();

    assert!(matches!(
        message.err().unwrap(),
        ValidationError::InvalidStrongParentsCount(0),
    ));
}

#[test]
fn invalid_no_parents_blocks() {
    let message = MessageBuilder::new()
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(Payload::from(
            IndexationPayload::new(rand_bytes(32), rand_bytes(256)).unwrap(),
        ))
        .with_nonce(0)
        .with_signature(rand_bytes_array())
        .finish();

    assert!(matches!(
        message.err().unwrap(),
        ValidationError::InvalidParentsBlocksCount(0),
    ));
}

#[test]
fn invalid_parents_blocks_more_than_max() {
    let message = MessageBuilder::new()
        .add_parents_block(
            ParentsBlock::new(ParentsKind::Strong, vec![MessageId::new(hex_decode(PARENT_1).unwrap())]).unwrap(),
        )
        .add_parents_block(
            ParentsBlock::new(ParentsKind::Weak, vec![MessageId::new(hex_decode(PARENT_2).unwrap())]).unwrap(),
        )
        .add_parents_block(
            ParentsBlock::new(ParentsKind::Liked, vec![MessageId::new(hex_decode(PARENT_3).unwrap())]).unwrap(),
        )
        .add_parents_block(
            ParentsBlock::new(
                ParentsKind::Disliked,
                vec![MessageId::new(hex_decode(PARENT_4).unwrap())],
            )
            .unwrap(),
        )
        .add_parents_block(
            ParentsBlock::new(ParentsKind::Strong, vec![MessageId::new(hex_decode(PARENT_5).unwrap())]).unwrap(),
        )
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(Payload::from(
            IndexationPayload::new(rand_bytes(32), rand_bytes(256)).unwrap(),
        ))
        .with_nonce(0)
        .with_signature(rand_bytes_array())
        .finish();

    assert!(matches!(
        message.err().unwrap(),
        ValidationError::InvalidParentsBlocksCount(5),
    ));
}

#[test]
fn packed_len() {
    let message = MessageBuilder::new()
        .add_parents_block(
            ParentsBlock::new(ParentsKind::Strong, vec![MessageId::new(hex_decode(PARENT_1).unwrap())]).unwrap(),
        )
        .add_parents_block(
            ParentsBlock::new(
                ParentsKind::Weak,
                vec![
                    MessageId::new(hex_decode(PARENT_2).unwrap()),
                    MessageId::new(hex_decode(PARENT_3).unwrap()),
                ],
            )
            .unwrap(),
        )
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

    assert_eq!(
        message.packed_len(),
        1 + 1 + 1 + 1 + 32 + 1 + 1 + 32 + 32 + 32 + 8 + 4 + 1 + 4 + 4 + 1 + 4 + 32 + 4 + 256 + 8 + 64,
    );
}

#[test]
fn packable_round_trip() {
    let message_a = MessageBuilder::new()
        .add_parents_block(
            ParentsBlock::new(ParentsKind::Strong, vec![MessageId::new(hex_decode(PARENT_1).unwrap())]).unwrap(),
        )
        .add_parents_block(
            ParentsBlock::new(
                ParentsKind::Weak,
                vec![
                    MessageId::new(hex_decode(PARENT_2).unwrap()),
                    MessageId::new(hex_decode(PARENT_3).unwrap()),
                ],
            )
            .unwrap(),
        )
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

    let message_b = Message::unpack_from_slice(message_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(message_a, message_b);
}

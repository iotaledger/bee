// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;
use bee_packable::{Packable, PackableExt};

use core::{ops::Deref, str::FromStr};

const MESSAGE_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn length() {
    assert_eq!(MessageId::LENGTH, 32);
}

#[test]
fn display_impl() {
    assert_eq!(format!("{}", MessageId::from_str(MESSAGE_ID).unwrap()), MESSAGE_ID);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", MessageId::from_str(MESSAGE_ID).unwrap()),
        "MessageId(".to_owned() + MESSAGE_ID + ")"
    );
}

#[test]
fn new_as_ref() {
    assert_eq!(
        MessageId::new([42; MessageId::LENGTH]).as_ref(),
        &[42; MessageId::LENGTH]
    );
}

#[test]
fn new_deref() {
    assert_eq!(
        MessageId::new([42; MessageId::LENGTH]).deref(),
        &[42; MessageId::LENGTH]
    );
}

#[test]
fn null_as_ref() {
    assert_eq!(MessageId::null().as_ref(), &[0; MessageId::LENGTH]);
}

#[test]
fn from_as_ref() {
    assert_eq!(
        MessageId::from([42; MessageId::LENGTH]).as_ref(),
        &[42; MessageId::LENGTH]
    );
}

#[test]
fn from_str_as_ref() {
    assert_eq!(
        MessageId::from_str(MESSAGE_ID).unwrap().as_ref(),
        &[
            0x52, 0xfd, 0xfc, 0x07, 0x21, 0x82, 0x65, 0x4f, 0x16, 0x3f, 0x5f, 0x0f, 0x9a, 0x62, 0x1d, 0x72, 0x95, 0x66,
            0xc7, 0x4d, 0x10, 0x03, 0x7c, 0x4d, 0x7b, 0xbb, 0x04, 0x07, 0xd1, 0xe2, 0xc6, 0x49
        ]
    );
}

#[test]
fn from_to_str() {
    assert_eq!(MESSAGE_ID, MessageId::from_str(MESSAGE_ID).unwrap().to_string());
}

#[test]
fn packed_len() {
    let message_id = MessageId::from_str(MESSAGE_ID).unwrap();

    assert_eq!(message_id.packed_len(), MessageId::LENGTH);
    assert_eq!(message_id.pack_to_vec().len(), MessageId::LENGTH);
}

#[test]
fn packable_round_trip() {
    let message_id_1 = MessageId::from_str(MESSAGE_ID).unwrap();
    let message_id_2 = MessageId::unpack_verified(message_id_1.pack_to_vec()).unwrap();

    assert_eq!(message_id_1, message_id_2);
}

#[test]
fn serde_round_trip() {
    let message_id_1 = MessageId::from_str(MESSAGE_ID).unwrap();
    let json = serde_json::to_string(&message_id_1).unwrap();
    let message_id_2 = serde_json::from_str::<MessageId>(&json).unwrap();

    assert_eq!(message_id_1, message_id_2);
}

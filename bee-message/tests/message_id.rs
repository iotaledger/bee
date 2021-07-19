// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;

use core::str::FromStr;

const MESSAGE_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const MESSAGE_ID_INVALID_HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64x";
const MESSAGE_ID_INVALID_LEN_TOO_SHORT: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";
const MESSAGE_ID_INVALID_LEN_TOO_LONG: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64900";

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
fn from_str_valid() {
    MessageId::from_str(MESSAGE_ID).unwrap();
}

#[test]
fn null() {
    assert_eq!(
        format!("{:?}", MessageId::null()),
        "MessageId(0000000000000000000000000000000000000000000000000000000000000000)"
    );
}

#[test]
fn from_str_invalid_hex() {
    assert!(matches!(
        MessageId::from_str(MESSAGE_ID_INVALID_HEX),
        Err(ValidationError::InvalidHexadecimalChar(hex))
            if hex == MESSAGE_ID_INVALID_HEX
    ));
}

#[test]
fn from_str_invalid_len_too_short() {
    assert!(matches!(
        MessageId::from_str(MESSAGE_ID_INVALID_LEN_TOO_SHORT),
        Err(ValidationError::InvalidHexadecimalLength(expected, actual))
            if expected == MESSAGE_ID_LENGTH * 2 && actual == MESSAGE_ID_LENGTH * 2 - 2
    ));
}

#[test]
fn from_str_invalid_len_too_long() {
    assert!(matches!(
        MessageId::from_str(MESSAGE_ID_INVALID_LEN_TOO_LONG),
        Err(ValidationError::InvalidHexadecimalLength(expected, actual))
            if expected == MESSAGE_ID_LENGTH * 2 && actual == MESSAGE_ID_LENGTH * 2 + 2
    ));
}

#[test]
fn from_to_str() {
    assert_eq!(MESSAGE_ID, MessageId::from_str(MESSAGE_ID).unwrap().to_string());
}

#[test]
fn packed_len() {
    let message_id = MessageId::from_str(MESSAGE_ID).unwrap();

    assert_eq!(message_id.packed_len(), 32);
    assert_eq!(message_id.pack_to_vec().unwrap().len(), 32);
}

#[test]
fn round_trip() {
    let message_id = MessageId::from_str(MESSAGE_ID).unwrap();
    let packed_message_id = message_id.pack_to_vec().unwrap();

    assert_eq!(packed_message_id.len(), message_id.packed_len());
    assert_eq!(message_id, MessageId::unpack_from_slice(packed_message_id).unwrap());
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::str::FromStr;

const MESSAGE_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const MESSAGE_ID_INVALID_HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64x";
const MESSAGE_ID_INVALID_LEN_TOO_SHORT: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";
const MESSAGE_ID_INVALID_LEN_TOO_LONG: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64900";

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", MessageId::from_str(MESSAGE_ID).unwrap()),
        "MessageId(52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649)"
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
        Err(Error::InvalidHexadecimalChar(hex))
            if hex == MESSAGE_ID_INVALID_HEX
    ));
}

#[test]
fn from_str_invalid_len_too_short() {
    assert!(matches!(
        MessageId::from_str(MESSAGE_ID_INVALID_LEN_TOO_SHORT),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == MESSAGE_ID_LENGTH * 2 && actual == MESSAGE_ID_LENGTH * 2 - 2
    ));
}

#[test]
fn from_str_invalid_len_too_long() {
    assert!(matches!(
        MessageId::from_str(MESSAGE_ID_INVALID_LEN_TOO_LONG),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == MESSAGE_ID_LENGTH * 2 && actual == MESSAGE_ID_LENGTH * 2 + 2
    ));
}

#[test]
fn from_to_str() {
    assert_eq!(MESSAGE_ID, MessageId::from_str(MESSAGE_ID).unwrap().to_string());
}

// Validate that the length of a packed `MessageId` matches the declared `packed_len()`.
#[test]
fn packed_len() {
    let message_id = MessageId::from_str(MESSAGE_ID).unwrap();

    assert_eq!(message_id.packed_len(), 32);
    assert_eq!(message_id.pack_new().len(), 32);
}

// Validate that a `unpack` ∘ `pack` round-trip results in the original message id.
#[test]
fn pack_unpack_valid() {
    let message_id = MessageId::from_str(MESSAGE_ID).unwrap();
    let packed_message_id = message_id.pack_new();

    assert_eq!(packed_message_id.len(), message_id.packed_len());
    assert_eq!(message_id, Packable::unpack(&mut packed_message_id.as_slice()).unwrap());
}

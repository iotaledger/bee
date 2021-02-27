// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use std::str::FromStr;

const MESSAGE_ID_VALID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const MESSAGE_ID_INVALID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";

#[test]
fn new_valid() {
    let message_id = MessageId::from_str(MESSAGE_ID_VALID).unwrap();
    let input = TreasuryInput::new(message_id);

    assert_eq!(*input.message_id(), message_id);
    assert_eq!(*input, message_id);
}

#[test]
fn from_valid() {
    let message_id = MessageId::from_str(MESSAGE_ID_VALID).unwrap();
    let input: TreasuryInput = message_id.into();

    assert_eq!(*input.message_id(), message_id);
    assert_eq!(*input, message_id);
}

#[test]
fn from_str_valid() {
    let message_id = MessageId::from_str(MESSAGE_ID_VALID).unwrap();
    let input = TreasuryInput::from_str(MESSAGE_ID_VALID).unwrap();

    assert_eq!(*input.message_id(), message_id);
    assert_eq!(*input, message_id);
}

#[test]
fn from_str_invalid() {
    assert!(matches!(
        TreasuryInput::from_str(MESSAGE_ID_INVALID),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == MESSAGE_ID_LENGTH * 2 && actual == MESSAGE_ID_LENGTH * 2 - 2
    ));
}

#[test]
fn from_str_to_str() {
    assert_eq!(
        TreasuryInput::from_str(MESSAGE_ID_VALID).unwrap().to_string(),
        MESSAGE_ID_VALID
    );
}

#[test]
fn packed_len() {
    let message_id = MessageId::from_str(MESSAGE_ID_VALID).unwrap();
    let input = TreasuryInput::new(message_id);

    assert_eq!(input.packed_len(), 32);
}

#[test]
fn pack_unpack_valid() {
    let input_1 = TreasuryInput::new(MessageId::from_str(MESSAGE_ID_VALID).unwrap());
    let input_2 = TreasuryInput::unpack(&mut input_1.pack_new().as_slice()).unwrap();

    assert_eq!(input_1, input_2);
}

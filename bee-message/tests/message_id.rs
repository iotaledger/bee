// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;

use core::str::FromStr;

const MESSAGE_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const MESSAGE_ID_INVALID_HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64x";
const MESSAGE_ID_INVALID_LEN: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";

#[test]
fn valid_from_str() {
    MessageId::from_str(MESSAGE_ID).unwrap();
}

#[test]
fn invalid_from_str_hex() {
    assert!(matches!(
        MessageId::from_str(MESSAGE_ID_INVALID_HEX),
        Err(Error::InvalidHexadecimalChar(hex))
            if hex == MESSAGE_ID_INVALID_HEX
    ));
}

#[test]
fn invalid_from_str_len() {
    assert!(matches!(
        MessageId::from_str(MESSAGE_ID_INVALID_LEN),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == MESSAGE_ID_LENGTH * 2 && actual == MESSAGE_ID_LENGTH * 2 - 2
    ));
}

#[test]
fn from_to_str() {
    assert_eq!(MESSAGE_ID, MessageId::from_str(MESSAGE_ID).unwrap().to_string());
}

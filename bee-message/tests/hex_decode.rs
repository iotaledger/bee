// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{error::ValidationError, util::hex_decode};

const HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const HEX_INVALID_CHAR: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64x";
const HEX_INVALID_LEN_TOO_SHORT: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6";
const HEX_INVALID_LEN_TOO_LONG: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64900";

#[test]
fn valid() {
    assert_eq!(
        hex_decode::<32>(HEX).unwrap(),
        [
            0x52, 0xfd, 0xfc, 0x07, 0x21, 0x82, 0x65, 0x4f, 0x16, 0x3f, 0x5f, 0x0f, 0x9a, 0x62, 0x1d, 0x72, 0x95, 0x66,
            0xc7, 0x4d, 0x10, 0x03, 0x7c, 0x4d, 0x7b, 0xbb, 0x04, 0x07, 0xd1, 0xe2, 0xc6, 0x49
        ]
    );
}

#[test]
fn invalid_char() {
    assert!(matches!(
        hex_decode::<32>(HEX_INVALID_CHAR),
        Err(ValidationError::InvalidHexadecimalChar(hex))
            if hex == HEX_INVALID_CHAR
    ));
}

#[test]
fn invalid_len_too_short() {
    assert!(matches!(
        hex_decode::<32>(HEX_INVALID_LEN_TOO_SHORT),
        Err(ValidationError::InvalidHexadecimalLength { expected, actual })
            if expected == 32 * 2 && actual == 32 * 2 - 2
    ));
}

#[test]
fn invalid_len_too_long() {
    assert!(matches!(
        hex_decode::<32>(HEX_INVALID_LEN_TOO_LONG),
        Err(ValidationError::InvalidHexadecimalLength { expected, actual })
            if expected == 32 * 2 && actual == 32 * 2 + 2
    ));
}

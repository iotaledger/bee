// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const OUTPUT_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00";
const INVALID_OUTPUT_ID_INDEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6497f00";
const INVALID_OUTPUT_ID_HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6497f0x";
const INVALID_OUTPUT_ID_LEN: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6497f";

#[test]
fn valid_new() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::new(transaction_id, 42).unwrap();

    assert_eq!(*output_id.transaction_id(), transaction_id);
    assert_eq!(output_id.index(), 42);
}

#[test]
fn valid_split() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::new(transaction_id, 42).unwrap();
    let (transaction_id_s, index) = output_id.split();

    assert_eq!(transaction_id_s, transaction_id);
    assert_eq!(index, 42);
}

#[test]
fn invalid_new() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();

    assert!(matches!(
        OutputId::new(transaction_id, 127),
        Err(Error::InvalidInputOutputIndex(127))
    ));
}

#[test]
fn valid_try_from() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id_bytes: [u8; OUTPUT_ID_LENGTH] = hex::decode(OUTPUT_ID).unwrap().try_into().unwrap();
    let output_id = OutputId::try_from(output_id_bytes).unwrap();

    assert_eq!(*output_id.transaction_id(), transaction_id);
    assert_eq!(output_id.index(), 42);
}

#[test]
fn invalid_try_from() {
    let output_id_bytes: [u8; OUTPUT_ID_LENGTH] = hex::decode(INVALID_OUTPUT_ID_INDEX).unwrap().try_into().unwrap();

    assert!(matches!(
        OutputId::try_from(output_id_bytes),
        Err(Error::InvalidInputOutputIndex(127))
    ));
}

#[test]
fn valid_from_str() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(*output_id.transaction_id(), transaction_id);
    assert_eq!(output_id.index(), 42);
}

#[test]
fn invalid_from_str_index() {
    assert!(matches!(
        OutputId::from_str(INVALID_OUTPUT_ID_INDEX),
        Err(Error::InvalidInputOutputIndex(127))
    ));
}

#[test]
fn invalid_from_str_hex() {
    assert!(matches!(
        OutputId::from_str(INVALID_OUTPUT_ID_HEX),
        Err(Error::InvalidHexadecimalChar(hex))
            if hex == INVALID_OUTPUT_ID_HEX
    ));
}

#[test]
fn invalid_from_str_len() {
    assert!(matches!(
        OutputId::from_str(INVALID_OUTPUT_ID_LEN),
        Err(Error::InvalidHexadecimalLength(expected, actual))
            if expected == OUTPUT_ID_LENGTH * 2 && actual == OUTPUT_ID_LENGTH * 2 - 2
    ));
}

#[test]
fn from_str_to_str() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(output_id.to_string(), OUTPUT_ID);
}

#[test]
fn pack_unpack_valid() {
    let output_id_1 = OutputId::from_str(OUTPUT_ID).unwrap();
    let output_id_2 = OutputId::unpack(&mut output_id_1.pack_new().as_slice()).unwrap();

    assert_eq!(output_id_1, output_id_2);
}

#[test]
fn pack_unpack_invalid() {
    let bytes = vec![
        82, 253, 252, 7, 33, 130, 101, 79, 22, 63, 95, 15, 154, 98, 29, 114, 149, 102, 199, 77, 16, 3, 124, 77, 123,
        187, 4, 7, 209, 226, 198, 73, 127, 0,
    ];

    assert!(matches!(
        OutputId::unpack(&mut bytes.as_slice()),
        Err(Error::InvalidInputOutputIndex(127))
    ));
}

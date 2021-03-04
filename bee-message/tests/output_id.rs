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
const OUTPUT_ID_INVALID_INDEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6497f00";
const OUTPUT_ID_INVALID_HEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6497f0x";
const OUTPUT_ID_INVALID_LEN: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6497f";

#[test]
fn new_valid() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::new(transaction_id, 42).unwrap();

    assert_eq!(*output_id.transaction_id(), transaction_id);
    assert_eq!(output_id.index(), 42);
}

#[test]
fn split_valid() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::new(transaction_id, 42).unwrap();
    let (transaction_id_s, index) = output_id.split();

    assert_eq!(transaction_id_s, transaction_id);
    assert_eq!(index, 42);
}

#[test]
fn new_invalid() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();

    assert!(matches!(
        OutputId::new(transaction_id, 127),
        Err(Error::InvalidInputOutputIndex(127))
    ));
}

#[test]
fn try_from_valid() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id_bytes: [u8; OUTPUT_ID_LENGTH] = hex::decode(OUTPUT_ID).unwrap().try_into().unwrap();
    let output_id = OutputId::try_from(output_id_bytes).unwrap();

    assert_eq!(*output_id.transaction_id(), transaction_id);
    assert_eq!(output_id.index(), 42);
}

#[test]
fn try_from_invalid() {
    let output_id_bytes: [u8; OUTPUT_ID_LENGTH] = hex::decode(OUTPUT_ID_INVALID_INDEX).unwrap().try_into().unwrap();

    assert!(matches!(
        OutputId::try_from(output_id_bytes),
        Err(Error::InvalidInputOutputIndex(127))
    ));
}

#[test]
fn from_str_valid() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(*output_id.transaction_id(), transaction_id);
    assert_eq!(output_id.index(), 42);
}

#[test]
fn from_str_invalid_index() {
    assert!(matches!(
        OutputId::from_str(OUTPUT_ID_INVALID_INDEX),
        Err(Error::InvalidInputOutputIndex(127))
    ));
}

#[test]
fn from_str_invalid_hex() {
    assert!(matches!(
        OutputId::from_str(OUTPUT_ID_INVALID_HEX),
        Err(Error::InvalidHexadecimalChar(hex))
            if hex == OUTPUT_ID_INVALID_HEX
    ));
}

#[test]
fn from_str_invalid_len() {
    assert!(matches!(
        OutputId::from_str(OUTPUT_ID_INVALID_LEN),
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
fn packed_len() {
    assert_eq!(OutputId::from_str(OUTPUT_ID).unwrap().packed_len(), 32 + 2);
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

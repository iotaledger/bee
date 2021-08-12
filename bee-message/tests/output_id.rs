// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::{MessageUnpackError, ValidationError},
    output::OutputId,
    payload::transaction::TransactionId,
    util::hex_decode,
};
use bee_packable::{Packable, UnpackError};

use core::{convert::TryFrom, str::FromStr};

const TRANSACTION_ID: &str = "d5c8b35f87a915c61f0d1b4af1f5d4a11b2bb4070d5c500074c74c752577b59c";
const OUTPUT_ID: &str = "d5c8b35f87a915c61f0d1b4af1f5d4a11b2bb4070d5c500074c74c752577b59c2a00";
const OUTPUT_ID_INVALID_INDEX: &str = "97517860f289cce53fdc7aab2442886147addc88633bcfb6f096e103ab30677d7f00";

#[test]
fn length() {
    assert_eq!(OutputId::LENGTH, 32 + 2);
}

#[test]
fn display_impl() {
    assert_eq!(format!("{}", OutputId::from_str(OUTPUT_ID).unwrap()), OUTPUT_ID);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", OutputId::from_str(OUTPUT_ID).unwrap()),
        "OutputId(".to_owned() + OUTPUT_ID + ")"
    );
}

#[test]
fn new_valid_getters() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::new(transaction_id, 42).unwrap();

    assert_eq!(output_id.transaction_id(), &transaction_id);
    assert_eq!(output_id.index(), 42);
}

#[test]
fn new_valid_split() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::new(transaction_id, 42).unwrap();
    let (transaction_id_s, index) = output_id.split();

    assert_eq!(transaction_id_s, transaction_id);
    assert_eq!(index, 42);
}

#[test]
fn new_invalid() {
    assert!(matches!(
        OutputId::new(TransactionId::from_str(TRANSACTION_ID).unwrap(), 127),
        Err(ValidationError::InvalidOutputIndex(127))
    ));
}

#[test]
fn try_from_valid() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::try_from(hex_decode(OUTPUT_ID).unwrap()).unwrap();

    assert_eq!(output_id.transaction_id(), &transaction_id);
    assert_eq!(output_id.index(), 42);
}

#[test]
fn try_from_invalid() {
    assert!(matches!(
        OutputId::try_from(hex_decode(OUTPUT_ID_INVALID_INDEX).unwrap()),
        Err(ValidationError::InvalidOutputIndex(127))
    ));
}

#[test]
fn from_str_valid() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(output_id.transaction_id(), &transaction_id);
    assert_eq!(output_id.index(), 42);
}

#[test]
fn from_str_invalid() {
    assert!(matches!(
        OutputId::from_str(OUTPUT_ID_INVALID_INDEX),
        Err(ValidationError::InvalidOutputIndex(127))
    ));
}

#[test]
fn from_str_to_str() {
    assert_eq!(OutputId::from_str(OUTPUT_ID).unwrap().to_string(), OUTPUT_ID);
}

#[test]
fn packed_len() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(output_id.packed_len(), 32 + 2);
    assert_eq!(output_id.pack_to_vec().unwrap().len(), 32 + 2);
}

#[test]
fn packable_round_trip() {
    let output_id_1 = OutputId::from_str(OUTPUT_ID).unwrap();
    let output_id_2 = OutputId::unpack_from_slice(output_id_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(output_id_1, output_id_2);
}

#[test]
fn unpack_invalid_index() {
    let bytes = vec![
        82, 253, 252, 7, 33, 130, 101, 79, 22, 63, 95, 15, 154, 98, 29, 114, 149, 102, 199, 77, 16, 3, 124, 77, 123,
        187, 4, 7, 209, 226, 198, 73, 127, 0,
    ];

    assert!(matches!(
        OutputId::unpack_from_slice(bytes).err().unwrap(),
        UnpackError::Packable(MessageUnpackError::Validation(ValidationError::InvalidOutputIndex(127))),
    ));
}

#[test]
fn serde_round_trip() {
    let output_id_1 = OutputId::from_str(OUTPUT_ID).unwrap();
    let json = serde_json::to_string(&output_id_1).unwrap();
    let output_id_2 = serde_json::from_str::<OutputId>(&json).unwrap();

    assert_eq!(output_id_1, output_id_2);
}

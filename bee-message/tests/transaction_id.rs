// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::transaction::TransactionId;
use bee_packable::Packable;

use core::{ops::Deref, str::FromStr};

const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn length() {
    assert_eq!(TransactionId::LENGTH, 32);
}

#[test]
fn display_impl() {
    assert_eq!(
        format!("{}", TransactionId::from_str(TRANSACTION_ID).unwrap()),
        TRANSACTION_ID
    );
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", TransactionId::from_str(TRANSACTION_ID).unwrap()),
        "TransactionId(".to_owned() + TRANSACTION_ID + ")"
    );
}

#[test]
fn new_as_ref() {
    assert_eq!(
        TransactionId::new([42; TransactionId::LENGTH]).as_ref(),
        &[42; TransactionId::LENGTH]
    );
}

#[test]
fn new_deref() {
    assert_eq!(
        TransactionId::new([42; TransactionId::LENGTH]).deref(),
        &[42; TransactionId::LENGTH]
    );
}

#[test]
fn from_as_ref() {
    assert_eq!(
        TransactionId::from([42; TransactionId::LENGTH]).as_ref(),
        &[42; TransactionId::LENGTH]
    );
}

#[test]
fn from_str_as_ref() {
    assert_eq!(
        TransactionId::from_str(TRANSACTION_ID).unwrap().as_ref(),
        &[
            0x52, 0xfd, 0xfc, 0x07, 0x21, 0x82, 0x65, 0x4f, 0x16, 0x3f, 0x5f, 0x0f, 0x9a, 0x62, 0x1d, 0x72, 0x95, 0x66,
            0xc7, 0x4d, 0x10, 0x03, 0x7c, 0x4d, 0x7b, 0xbb, 0x04, 0x07, 0xd1, 0xe2, 0xc6, 0x49
        ]
    );
}

#[test]
fn from_to_str() {
    assert_eq!(
        TRANSACTION_ID,
        TransactionId::from_str(TRANSACTION_ID).unwrap().to_string()
    );
}

#[test]
fn packed_len() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();

    assert_eq!(transaction_id.packed_len(), TransactionId::LENGTH);
    assert_eq!(transaction_id.pack_to_vec().unwrap().len(), TransactionId::LENGTH);
}

#[test]
fn packable_round_trip() {
    let transaction_id_1 = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let transaction_id_2 = TransactionId::unpack_from_slice(transaction_id_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(transaction_id_1, transaction_id_2);
}

#[test]
fn serde_round_trip() {
    let transaction_id_1 = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let json = serde_json::to_string(&transaction_id_1).unwrap();
    let transaction_id_2 = serde_json::from_str::<TransactionId>(&json).unwrap();

    assert_eq!(transaction_id_1, transaction_id_2);
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::transaction::TransactionId;

use packable::PackableExt;

use core::str::FromStr;

const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", TransactionId::from_str(TRANSACTION_ID).unwrap()),
        "TransactionId(52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649)"
    );
}

#[test]
fn from_str_valid() {
    TransactionId::from_str(TRANSACTION_ID).unwrap();
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

    assert_eq!(transaction_id.packed_len(), 32);
    assert_eq!(transaction_id.pack_to_vec().len(), 32);
}

#[test]
fn pack_unpack_valid() {
    let transaction_id = TransactionId::from_str(TRANSACTION_ID).unwrap();
    let packed_transaction_id = transaction_id.pack_to_vec();

    assert_eq!(
        transaction_id,
        PackableExt::unpack_verified(&mut packed_transaction_id.as_slice()).unwrap()
    );
}

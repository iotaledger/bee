// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::MessageUnpackError,
    input::{Input, InputUnpackError, UtxoInput},
    output::OutputId,
    payload::transaction::TransactionId,
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::rand_bytes;

#[test]
fn from_utxo() {
    let utxo_input = UtxoInput::from(OutputId::new(TransactionId::new([42; TransactionId::LENGTH]), 0).unwrap());
    let input = Input::from(utxo_input.clone());

    assert_eq!(input.kind(), 0);
    assert!(matches!(input, Input::Utxo(input) if {input == utxo_input}));
}

#[test]
fn packed_len() {
    let input = Input::from(UtxoInput::from(
        OutputId::new(TransactionId::new([42; TransactionId::LENGTH]), 0).unwrap(),
    ));

    assert_eq!(input.packed_len(), 1 + 32 + 2);
    assert_eq!(input.pack_to_vec().unwrap().len(), 1 + 32 + 2);
}

#[test]
fn packable_round_trip() {
    let input_1 = Input::from(UtxoInput::from(
        OutputId::new(TransactionId::new([42; TransactionId::LENGTH]), 0).unwrap(),
    ));
    let input_2 = Input::unpack_from_slice(input_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(input_1, input_2);
}

#[test]
fn unpack_invalid_tag() {
    let mut bytes = vec![1];
    bytes.extend(rand_bytes(32));
    bytes.extend(vec![0, 0]);

    assert!(matches!(
        Input::unpack_from_slice(bytes),
        Err(UnpackError::Packable(MessageUnpackError::Input(
            InputUnpackError::InvalidKind(1)
        ))),
    ));
}

#[test]
fn serde_round_trip() {
    let input_1 = Input::from(UtxoInput::from(
        OutputId::new(TransactionId::new([42; TransactionId::LENGTH]), 0).unwrap(),
    ));
    let json = serde_json::to_string(&input_1).unwrap();
    let input_2 = serde_json::from_str::<Input>(&json).unwrap();

    assert_eq!(input_1, input_2);
}

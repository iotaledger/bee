// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::{
    address::{Address, Ed25519Address},
    constants::IOTA_SUPPLY,
    input::{Input, TreasuryInput, UtxoInput},
    output::{Output, SimpleOutput, TreasuryOutput},
    payload::TreasuryTransactionPayload,
    Error,
};

use core::str::FromStr;

const MESSAGE_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const UTXO_INPUT: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00";
const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(TreasuryTransactionPayload::KIND, 4);
}

#[test]
fn new_valid() {
    let input = Input::from(TreasuryInput::from_str(MESSAGE_ID).unwrap());
    let output = Output::from(TreasuryOutput::new(1_000).unwrap());
    let transaction = TreasuryTransactionPayload::new(input.clone(), output.clone()).unwrap();

    assert_eq!(*transaction.input(), input);
    assert_eq!(*transaction.output(), output);
}

#[test]
fn new_invalid_input() {
    let input = Input::from(UtxoInput::from_str(UTXO_INPUT).unwrap());
    let output = Output::from(TreasuryOutput::new(1_000).unwrap());

    assert!(matches!(
        TreasuryTransactionPayload::new(input.clone(), output),
        Err(Error::InvalidInputKind(k)) if k == input.kind()
    ));
}

#[test]
fn new_invalid_output() {
    let input = Input::from(TreasuryInput::from_str(MESSAGE_ID).unwrap());
    let output = Output::from(
        SimpleOutput::new(
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            IOTA_SUPPLY,
        )
        .unwrap(),
    );

    assert!(matches!(
        TreasuryTransactionPayload::new(input, output.clone()),
        Err(Error::InvalidOutputKind(k)) if k == output.kind()
    ));
}

#[test]
fn packed_len() {
    let treasury_transaction = TreasuryTransactionPayload::new(
        Input::from(TreasuryInput::from_str(MESSAGE_ID).unwrap()),
        Output::from(TreasuryOutput::new(1_000).unwrap()),
    )
    .unwrap();

    assert_eq!(treasury_transaction.packed_len(), 1 + 32 + 1 + 8);
    assert_eq!(treasury_transaction.pack_new().len(), 1 + 32 + 1 + 8);
}

#[test]
fn pack_unpack_valid() {
    let transaction_1 = TreasuryTransactionPayload::new(
        Input::from(TreasuryInput::from_str(MESSAGE_ID).unwrap()),
        Output::from(TreasuryOutput::new(1_000).unwrap()),
    )
    .unwrap();
    let transaction_2 = TreasuryTransactionPayload::unpack(&mut transaction_1.pack_new().as_slice()).unwrap();

    assert_eq!(transaction_1, transaction_2);
}

#[test]
fn pack_unpack_invalid() {
    let bytes = vec![
        1, 82, 253, 252, 7, 33, 130, 101, 79, 22, 63, 95, 15, 154, 98, 29, 114, 149, 102, 199, 77, 16, 3, 124, 77, 123,
        187, 4, 7, 209, 226, 198, 73, // Faulty byte here ->
        0, 232, 3, 0, 0, 0, 0, 0, 0,
    ];

    // Actual error is not checked because the output type check is done after the output is parsed so the error is not
    // `InvalidOutputKind` but something related to an invalid address, so not really relevant for this test.
    assert!(TreasuryTransactionPayload::unpack(&mut bytes.as_slice()).is_err());
}

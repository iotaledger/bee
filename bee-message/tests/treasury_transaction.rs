// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use std::str::FromStr;

const MESSAGE_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const UTXO_INPUT: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00";
const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn valid() {
    let input = Input::from(TreasuryInput::from_str(MESSAGE_ID).unwrap());
    let output = Output::from(TreasuryOutput::new(1_000).unwrap());
    let transaction = TreasuryTransactionPayload::new(input.clone(), output.clone()).unwrap();

    assert_eq!(*transaction.input(), input);
    assert_eq!(*transaction.output(), output);
}

#[test]
fn invalid_input() {
    let input = Input::from(UTXOInput::from_str(UTXO_INPUT).unwrap());
    let output = Output::from(TreasuryOutput::new(1_000).unwrap());

    assert!(matches!(
        TreasuryTransactionPayload::new(input.clone(), output),
        Err(Error::InvalidInputKind(k)) if k == input.kind()
    ));
}

#[test]
fn invalid_output() {
    let input = Input::from(TreasuryInput::from_str(MESSAGE_ID).unwrap());
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let output = Output::from(SignatureLockedSingleOutput::new(address, IOTA_SUPPLY).unwrap());

    assert!(matches!(
        TreasuryTransactionPayload::new(input, output.clone()),
        Err(Error::InvalidOutputKind(k)) if k == output.kind()
    ));
}

#[test]
fn pack_unpack() {
    let input = Input::from(TreasuryInput::from_str(MESSAGE_ID).unwrap());
    let output = Output::from(TreasuryOutput::new(1_000).unwrap());
    let transaction_1 = TreasuryTransactionPayload::new(input, output).unwrap();
    let transaction_2 = TreasuryTransactionPayload::unpack(&mut transaction_1.pack_new().as_slice()).unwrap();

    assert_eq!(transaction_1, transaction_2);
}

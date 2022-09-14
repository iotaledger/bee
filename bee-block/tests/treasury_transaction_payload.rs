// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;

use bee_block::{
    input::TreasuryInput, output::TreasuryOutput, payload::TreasuryTransactionPayload, protocol::protocol_parameters,
};
use packable::PackableExt;

const BLOCK_ID: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(TreasuryTransactionPayload::KIND, 4);
}

#[test]
fn new_valid() {
    let input = TreasuryInput::from_str(BLOCK_ID).unwrap();
    let output = TreasuryOutput::new(1_000, protocol_parameters().token_supply()).unwrap();
    let transaction = TreasuryTransactionPayload::new(input, output.clone()).unwrap();

    assert_eq!(*transaction.input(), input);
    assert_eq!(*transaction.output(), output);
}

#[test]
fn packed_len() {
    let treasury_transaction = TreasuryTransactionPayload::new(
        TreasuryInput::from_str(BLOCK_ID).unwrap(),
        TreasuryOutput::new(1_000, protocol_parameters().token_supply()).unwrap(),
    )
    .unwrap();

    assert_eq!(treasury_transaction.packed_len(), 1 + 32 + 1 + 8);
    assert_eq!(treasury_transaction.pack_to_vec().len(), 1 + 32 + 1 + 8);
}

#[test]
fn pack_unpack_valid() {
    let protocol_parameters = protocol_parameters();
    let transaction_1 = TreasuryTransactionPayload::new(
        TreasuryInput::from_str(BLOCK_ID).unwrap(),
        TreasuryOutput::new(1_000, protocol_parameters.token_supply()).unwrap(),
    )
    .unwrap();
    let transaction_2 =
        TreasuryTransactionPayload::unpack_verified(&mut transaction_1.pack_to_vec().as_slice(), &protocol_parameters)
            .unwrap();

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
    assert!(TreasuryTransactionPayload::unpack_verified(&mut bytes.as_slice(), &protocol_parameters()).is_err());
}

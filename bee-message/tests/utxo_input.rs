// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{input::UtxoInput, output::OutputId};
use bee_packable::Packable;

use core::str::FromStr;

const OUTPUT_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00";

#[test]
fn kind() {
    assert_eq!(UtxoInput::KIND, 0);
}

#[test]
fn display_impl() {
    assert_eq!(
        format!("{}", UtxoInput::new(OutputId::from_str(OUTPUT_ID).unwrap())),
        OUTPUT_ID
    );
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", UtxoInput::new(OutputId::from_str(OUTPUT_ID).unwrap())),
        "UtxoInput(OutputId(".to_owned() + OUTPUT_ID + "))"
    );
}

#[test]
fn new() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(UtxoInput::new(output_id).output_id(), &output_id);
}

#[test]
fn from() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(UtxoInput::from(output_id).output_id(), &output_id);
}

#[test]
fn deref() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();
    let utxo_input = UtxoInput::new(output_id);

    assert_eq!(utxo_input.transaction_id(), output_id.transaction_id());
    assert_eq!(utxo_input.index(), output_id.index());
}

#[test]
fn packed_len() {
    let input = UtxoInput::new(OutputId::from_str(OUTPUT_ID).unwrap());

    assert_eq!(input.packed_len(), 32 + 2);
    assert_eq!(input.pack_to_vec().len(), 32 + 2);
}

#[test]
fn packable_round_trip() {
    let input_1 = UtxoInput::new(OutputId::from_str(OUTPUT_ID).unwrap());
    let input_2 = UtxoInput::unpack_from_slice(input_1.pack_to_vec()).unwrap();

    assert_eq!(input_1, input_2);
}

#[test]
fn serde_round_trip() {
    let utxo_input_1 = UtxoInput::new(OutputId::from_str(OUTPUT_ID).unwrap());
    let json = serde_json::to_string(&utxo_input_1).unwrap();
    let utxo_input_2 = serde_json::from_str::<UtxoInput>(&json).unwrap();

    assert_eq!(utxo_input_1, utxo_input_2);
}

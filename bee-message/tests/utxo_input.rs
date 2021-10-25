// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::PackableExt;

use core::str::FromStr;

const OUTPUT_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00";

#[test]
fn kind() {
    assert_eq!(UtxoInput::KIND, 0);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", UtxoInput::from_str(OUTPUT_ID).unwrap()),
        "UtxoInput(52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00)"
    );
}

#[test]
fn new_valid() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();
    let input = UtxoInput::new(*output_id.transaction_id(), output_id.index()).unwrap();

    assert_eq!(*input.output_id(), output_id);
}

#[test]
fn from_valid() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();
    let input: UtxoInput = output_id.into();

    assert_eq!(*input.output_id(), output_id);
}

#[test]
fn from_str_valid() {
    assert_eq!(
        *UtxoInput::from_str(OUTPUT_ID).unwrap().output_id(),
        OutputId::from_str(OUTPUT_ID).unwrap()
    );
}

#[test]
fn from_str_to_str() {
    assert_eq!(UtxoInput::from_str(OUTPUT_ID).unwrap().to_string(), OUTPUT_ID);
}

#[test]
fn packed_len() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(
        UtxoInput::new(*output_id.transaction_id(), output_id.index())
            .unwrap()
            .packed_len(),
        32 + 2
    );
    assert_eq!(output_id.pack_to_vec().len(), 32 + 2);
}

#[test]
fn pack_unpack() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();
    let input_1 = UtxoInput::new(*output_id.transaction_id(), output_id.index()).unwrap();
    let input_2 = UtxoInput::unpack_verified(&mut input_1.pack_to_vec().as_slice()).unwrap();

    assert_eq!(input_1, input_2);
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;

use core::str::FromStr;

const OUTPUT_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00";
const OUTPUT_ID_INVALID_INDEX: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6497f00";
const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(UtxoInput::KIND, 0);
}

#[test]
fn display_impl() {
    assert_eq!(format!("{}", UtxoInput::from_str(OUTPUT_ID).unwrap()), OUTPUT_ID);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", UtxoInput::from_str(OUTPUT_ID).unwrap()),
        "UtxoInput(".to_owned() + OUTPUT_ID + ")"
    );
}

#[test]
fn new_valid() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(
        UtxoInput::new(*output_id.transaction_id(), output_id.index())
            .unwrap()
            .output_id(),
        &output_id
    );
}

#[test]
fn new_invalid() {
    assert!(matches!(
        UtxoInput::new(TransactionId::from_str(TRANSACTION_ID).unwrap(), 127),
        Err(ValidationError::InvalidOutputIndex(127))
    ));
}

#[test]
fn from_valid() {
    let output_id = OutputId::from_str(OUTPUT_ID).unwrap();

    assert_eq!(UtxoInput::from(output_id).output_id(), &output_id);
}

#[test]
fn from_str_valid() {
    assert_eq!(
        *UtxoInput::from_str(OUTPUT_ID).unwrap().output_id(),
        OutputId::from_str(OUTPUT_ID).unwrap()
    );
}

#[test]
fn from_str_invalid() {
    assert!(matches!(
        UtxoInput::from_str(OUTPUT_ID_INVALID_INDEX),
        Err(ValidationError::InvalidOutputIndex(127))
    ));
}

#[test]
fn from_str_to_str() {
    assert_eq!(UtxoInput::from_str(OUTPUT_ID).unwrap().to_string(), OUTPUT_ID);
}

#[test]
fn packed_len() {
    let input = UtxoInput::from_str(OUTPUT_ID).unwrap();

    assert_eq!(input.packed_len(), 32 + 2);
    assert_eq!(input.pack_to_vec().unwrap().len(), 32 + 2);
}

#[test]
fn packable_round_trip() {
    let input_1 = UtxoInput::from_str(OUTPUT_ID).unwrap();
    let input_2 = UtxoInput::unpack_from_slice(input_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(input_1, input_2);
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{output::TreasuryOutput, protocol::protocol_parameters, Error};
use packable::{error::UnpackError, PackableExt};

#[test]
fn kind() {
    assert_eq!(TreasuryOutput::KIND, 2);
}

#[test]
fn new_valid_min_amount() {
    assert_eq!(
        TreasuryOutput::new(0, protocol_parameters().token_supply())
            .unwrap()
            .amount(),
        0
    );
}

#[test]
fn new_valid_max_amount() {
    let protocol_parameters = protocol_parameters();

    assert_eq!(
        TreasuryOutput::new(protocol_parameters.token_supply(), protocol_parameters.token_supply())
            .unwrap()
            .amount(),
        protocol_parameters.token_supply()
    );
}

#[test]
fn invalid_more_than_max_amount() {
    assert!(matches!(
        TreasuryOutput::new(3_038_287_259_199_220_266, protocol_parameters().token_supply()),
        Err(Error::InvalidTreasuryOutputAmount(3_038_287_259_199_220_266))
    ));
}

#[test]
fn packed_len() {
    let treasury_output = TreasuryOutput::new(1_000, protocol_parameters().token_supply()).unwrap();

    assert_eq!(treasury_output.packed_len(), 8);
    assert_eq!(treasury_output.pack_to_vec().len(), 8);
}

#[test]
fn pack_unpack_valid() {
    let output_1 = TreasuryOutput::new(1_000, protocol_parameters().token_supply()).unwrap();
    let output_2 = TreasuryOutput::unpack_verified(output_1.pack_to_vec().as_slice(), &protocol_parameters()).unwrap();

    assert_eq!(output_1, output_2);
}

#[test]
fn pack_unpack_invalid() {
    assert!(matches!(
        TreasuryOutput::unpack_verified(
            vec![0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a].as_slice(),
            &protocol_parameters()
        ),
        Err(UnpackError::Packable(Error::InvalidTreasuryOutputAmount(
            3_038_287_259_199_220_266
        )))
    ));
}

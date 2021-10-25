// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::{bounded::InvalidBoundedU64, error::UnpackError, PackableExt};

#[test]
fn kind() {
    assert_eq!(TreasuryOutput::KIND, 2);
}

#[test]
fn new_valid_min_amount() {
    assert_eq!(TreasuryOutput::new(0).unwrap().amount(), 0);
}

#[test]
fn new_valid_max_amount() {
    assert_eq!(TreasuryOutput::new(IOTA_SUPPLY).unwrap().amount(), IOTA_SUPPLY);
}

#[test]
fn invalid_more_than_max_amount() {
    assert!(matches!(
        TreasuryOutput::new(3_038_287_259_199_220_266),
        Err(Error::InvalidTreasuryAmount(InvalidBoundedU64(
            3_038_287_259_199_220_266
        )))
    ));
}

#[test]
fn packed_len() {
    let treasury_output = TreasuryOutput::new(1_000).unwrap();

    assert_eq!(treasury_output.packed_len(), 8);
    assert_eq!(treasury_output.pack_to_vec().len(), 8);
}

#[test]
fn pack_unpack_valid() {
    let output_1 = TreasuryOutput::new(1_000).unwrap();
    let output_2 = TreasuryOutput::unpack_verified(&mut output_1.pack_to_vec().as_slice()).unwrap();

    assert_eq!(output_1, output_2);
}

#[test]
fn pack_unpack_invalid() {
    assert!(matches!(
        TreasuryOutput::unpack_verified(&mut vec![0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a].as_slice()),
        Err(UnpackError::Packable(Error::InvalidTreasuryAmount(InvalidBoundedU64(
            3_038_287_259_199_220_266
        ))))
    ));
}

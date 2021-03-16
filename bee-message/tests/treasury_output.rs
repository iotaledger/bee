// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

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
        Err(Error::InvalidTreasuryAmount(3_038_287_259_199_220_266))
    ));
}

#[test]
fn packed_len() {
    assert_eq!(TreasuryOutput::new(1_000).unwrap().packed_len(), 8);
}

#[test]
fn pack_unpack_valid() {
    let output_1 = TreasuryOutput::new(1_000).unwrap();
    let output_2 = TreasuryOutput::unpack(&mut output_1.pack_new().as_slice()).unwrap();

    assert_eq!(output_1, output_2);
}

#[test]
fn pack_unpack_invalid() {
    assert!(matches!(
        TreasuryOutput::unpack(&mut vec![0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a].as_slice()),
        Err(Error::InvalidTreasuryAmount(3_038_287_259_199_220_266))
    ));
}

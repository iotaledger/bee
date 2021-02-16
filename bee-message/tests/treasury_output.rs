// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

#[test]
fn new_valid_min_amount() {
    let output = TreasuryOutput::new(1).unwrap();

    assert_eq!(output.amount(), 1);
}

#[test]
fn new_valid_max_amount() {
    let output = TreasuryOutput::new(IOTA_SUPPLY).unwrap();

    assert_eq!(output.amount(), IOTA_SUPPLY);
}

#[test]
fn new_invalid_less_than_min_amount() {
    assert!(matches!(TreasuryOutput::new(0), Err(Error::InvalidTreasuryAmount(0))));
}

#[test]
fn invalid_more_than_max_amount() {
    assert!(matches!(
        TreasuryOutput::new(3_333_333_333_333_333),
        Err(Error::InvalidTreasuryAmount(3_333_333_333_333_333))
    ));
}

#[test]
fn pack_unpack_valid() {
    let output_1 = TreasuryOutput::new(1_000).unwrap();
    let output_2 = TreasuryOutput::unpack(&mut output_1.pack_new().as_slice()).unwrap();

    assert_eq!(output_1, output_2);
}

#[test]
fn pack_unpack_invalid() {
    let bytes = vec![0, 0, 0, 0, 0, 0, 0, 0];

    assert!(matches!(
        TreasuryOutput::unpack(&mut bytes.as_slice()),
        Err(Error::InvalidTreasuryAmount(0))
    ));
}

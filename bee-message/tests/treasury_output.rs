// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

#[test]
fn valid() {
    let output = TreasuryOutput::new(1_000).unwrap();

    assert_eq!(output.amount(), 1_000);
}

#[test]
fn valid_supply_amount() {
    let output = TreasuryOutput::new(IOTA_SUPPLY).unwrap();

    assert_eq!(output.amount(), IOTA_SUPPLY);
}

#[test]
fn invalid_null_amount() {
    assert!(matches!(TreasuryOutput::new(0), Err(Error::InvalidTreasuryAmount(0))));
}

#[test]
fn invalid_big_amount() {
    assert!(matches!(
        TreasuryOutput::new(3_333_333_333_333_333),
        Err(Error::InvalidTreasuryAmount(3_333_333_333_333_333))
    ));
}

#[test]
fn pack_unpack() {
    let output_1 = TreasuryOutput::new(1_000).unwrap();
    let output_2 = TreasuryOutput::unpack(&mut output_1.pack_new().as_slice()).unwrap();

    assert_eq!(output_1, output_2);
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use std::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn valid() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let output = SignatureLockedDustAllowanceOutput::new(address, 1_000_000).unwrap();

    assert_eq!(*output.address(), address);
    assert_eq!(output.amount(), 1_000_000);
}

#[test]
fn valid_supply_amount() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let output = SignatureLockedDustAllowanceOutput::new(address, IOTA_SUPPLY).unwrap();

    assert_eq!(*output.address(), address);
    assert_eq!(output.amount(), IOTA_SUPPLY);
}

#[test]
fn invalid_amount_less_than_minimum() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    assert!(matches!(
        SignatureLockedDustAllowanceOutput::new(address, 999_999),
        Err(Error::InvalidDustAmount(999_999))
    ));
}

#[test]
fn invalid_big_amount() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    assert!(matches!(
        SignatureLockedDustAllowanceOutput::new(address, 3_333_333_333_333_333),
        Err(Error::InvalidDustAmount(3_333_333_333_333_333))
    ));
}

#[test]
fn pack_unpack() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let output_1 = SignatureLockedDustAllowanceOutput::new(address, 1_000_000).unwrap();
    let output_2 = SignatureLockedDustAllowanceOutput::unpack(&mut output_1.pack_new().as_slice()).unwrap();

    assert_eq!(output_1, output_2);
}

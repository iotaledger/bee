// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(SignatureLockedSingleOutput::KIND, 0);
}

#[test]
fn new_valid_min_amount() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let output = SignatureLockedSingleOutput::new(address, 1).unwrap();

    assert_eq!(*output.address(), address);
    assert_eq!(output.amount(), 1);
}

#[test]
fn new_valid_max_amount() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let output = SignatureLockedSingleOutput::new(address, IOTA_SUPPLY).unwrap();

    assert_eq!(*output.address(), address);
    assert_eq!(output.amount(), IOTA_SUPPLY);
}

#[test]
fn new_invalid_less_than_min_amount() {
    assert!(matches!(
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 0),
        Err(Error::InvalidAmount(0))
    ));
}

#[test]
fn new_invalid_more_than_max_amount() {
    assert!(matches!(
        SignatureLockedSingleOutput::new(
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            3_333_333_333_333_333
        ),
        Err(Error::InvalidAmount(3_333_333_333_333_333))
    ));
}

#[test]
fn packed_len() {
    assert_eq!(
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 1)
            .unwrap()
            .packed_len(),
        1 + 32 + 8
    );
}

#[test]
fn pack_unpack_valid() {
    let output_1 =
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 1_000)
            .unwrap();
    let output_2 = SignatureLockedSingleOutput::unpack(&mut output_1.pack_new().as_slice()).unwrap();

    assert_eq!(output_1, output_2);
}

#[test]
fn pack_unpack_invalid() {
    assert!(matches!(
        SignatureLockedSingleOutput::unpack(
            &mut vec![
                0, 82, 253, 252, 7, 33, 130, 101, 79, 22, 63, 95, 15, 154, 98, 29, 114, 149, 102, 199, 77, 16, 3, 124,
                77, 123, 187, 4, 7, 209, 226, 198, 73, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .as_slice()
        ),
        Err(Error::InvalidAmount(0))
    ));
}

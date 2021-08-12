// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, Ed25519Address},
    error::{MessageUnpackError, ValidationError},
    output::SignatureLockedSingleOutput,
    IOTA_SUPPLY,
};
use bee_packable::{Packable, UnpackError};

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(SignatureLockedSingleOutput::KIND, 0);
}

#[test]
fn new_valid_min_amount() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let output = SignatureLockedSingleOutput::new(address.clone(), 1).unwrap();

    assert_eq!(output.address(), &address);
    assert_eq!(output.amount(), 1);
}

#[test]
fn new_valid_max_amount() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let output = SignatureLockedSingleOutput::new(address.clone(), IOTA_SUPPLY).unwrap();

    assert_eq!(output.address(), &address);
    assert_eq!(output.amount(), IOTA_SUPPLY);
}

#[test]
fn new_invalid_less_than_min_amount() {
    assert!(matches!(
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 0),
        Err(ValidationError::InvalidAmount(0))
    ));
}

#[test]
fn new_invalid_more_than_max_amount() {
    assert!(matches!(
        SignatureLockedSingleOutput::new(
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            3_333_333_333_333_333
        ),
        Err(ValidationError::InvalidAmount(3_333_333_333_333_333))
    ));
}

#[test]
fn unpack_invalid_amount() {
    assert!(matches!(
        SignatureLockedSingleOutput::unpack_from_slice(vec![
            0, 82, 253, 252, 7, 33, 130, 101, 79, 22, 63, 95, 15, 154, 98, 29, 114, 149, 102, 199, 77, 16, 3, 124, 77,
            123, 187, 4, 7, 209, 226, 198, 73, 0, 0, 0, 0, 0, 0, 0, 0,
        ])
        .err()
        .unwrap(),
        UnpackError::Packable(MessageUnpackError::Validation(ValidationError::InvalidAmount(0))),
    ));
}

#[test]
fn packed_len() {
    let output =
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 1).unwrap();

    assert_eq!(output.packed_len(), 1 + 32 + 8);
    assert_eq!(output.pack_to_vec().unwrap().len(), 1 + 32 + 8);
}

#[test]
fn packable_round_trip() {
    let output_1 =
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 1_000)
            .unwrap();
    let output_2 = SignatureLockedSingleOutput::unpack_from_slice(output_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(output_1, output_2);
}

#[test]
fn serde_round_trip() {
    let signature_locked_single_output_1 =
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 1).unwrap();
    let json = serde_json::to_string(&signature_locked_single_output_1).unwrap();
    let signature_locked_single_output_2 = serde_json::from_str::<SignatureLockedSingleOutput>(&json).unwrap();

    assert_eq!(signature_locked_single_output_1, signature_locked_single_output_2);
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;
use bee_test::rand::bytes::rand_bytes_array;

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(SignatureLockedAssetAllowanceOutput::KIND, 1);
}

#[test]
fn new_valid() {
    let output = SignatureLockedAssetAllowanceOutput::new(
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        vec![
            AssetBalance::new(rand_bytes_array(), 1000),
            AssetBalance::new(rand_bytes_array(), 1000),
            AssetBalance::new(rand_bytes_array(), 1000),
        ],
    );

    assert!(output.is_ok());
}

#[test]
fn packed_len() {
    let output = SignatureLockedAssetAllowanceOutput::new(
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        vec![
            AssetBalance::new(rand_bytes_array(), 1000),
            AssetBalance::new(rand_bytes_array(), 1000),
            AssetBalance::new(rand_bytes_array(), 1000),
        ],
    )
    .unwrap();

    assert_eq!(output.packed_len(), 1 + 32 + 4 + 3 * (32 + 8));
    assert_eq!(output.pack_to_vec().unwrap().len(), 1 + 32 + 4 + 3 * (32 + 8));
}

#[test]
fn round_trip() {
    let output_a = SignatureLockedAssetAllowanceOutput::new(
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        vec![
            AssetBalance::new(rand_bytes_array(), 1000),
            AssetBalance::new(rand_bytes_array(), 1000),
            AssetBalance::new(rand_bytes_array(), 1000),
        ],
    )
    .unwrap();

    let output_b = SignatureLockedAssetAllowanceOutput::unpack_from_slice(output_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(output_a, output_b);
}

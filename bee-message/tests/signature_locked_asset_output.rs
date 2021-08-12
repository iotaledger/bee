// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, Ed25519Address},
    output::{AssetBalance, AssetId, SignatureLockedAssetOutput},
};
use bee_packable::Packable;
use bee_test::rand::bytes::rand_bytes_array;

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn kind() {
    assert_eq!(SignatureLockedAssetOutput::KIND, 1);
}

#[test]
fn new_valid() {
    let output = SignatureLockedAssetOutput::new(
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        vec![
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
        ],
    );

    assert!(output.is_ok());
}

#[test]
fn accessors_eq() {
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let balances = vec![
        AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
        AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
        AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
    ];

    let output = SignatureLockedAssetOutput::new(address.clone(), balances.clone()).unwrap();

    assert_eq!(output.address(), &address);
    assert_eq!(output.balance_iter().cloned().collect::<Vec<AssetBalance>>(), balances);
}

#[test]
fn asset_balance_accessors_eq() {
    let id = AssetId::new(rand_bytes_array());
    let amount = 1000;

    let balance = AssetBalance::new(id.clone(), amount);

    assert_eq!(*balance.id(), id);
    assert_eq!(balance.balance(), amount);
}

#[test]
fn packed_len() {
    let output = SignatureLockedAssetOutput::new(
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        vec![
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
        ],
    )
    .unwrap();

    assert_eq!(output.packed_len(), 1 + 32 + 4 + 3 * (32 + 8));
    assert_eq!(output.pack_to_vec().unwrap().len(), 1 + 32 + 4 + 3 * (32 + 8));
}

#[test]
fn packable_round_trip() {
    let output_a = SignatureLockedAssetOutput::new(
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        vec![
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
        ],
    )
    .unwrap();

    let output_b = SignatureLockedAssetOutput::unpack_from_slice(output_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(output_a, output_b);
}

#[test]
fn serde_round_trip() {
    let signature_locked_asset_output_1 = SignatureLockedAssetOutput::new(
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        vec![
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
            AssetBalance::new(AssetId::new(rand_bytes_array()), 1000),
        ],
    )
    .unwrap();
    let json = serde_json::to_string(&signature_locked_asset_output_1).unwrap();
    let signature_locked_asset_output_2 = serde_json::from_str::<SignatureLockedAssetOutput>(&json).unwrap();

    assert_eq!(signature_locked_asset_output_1, signature_locked_asset_output_2);
}

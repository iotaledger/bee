// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, Ed25519Address},
    error::MessageUnpackError,
    output::{
        AssetBalance, AssetId, Output, OutputUnpackError, SignatureLockedAssetOutput, SignatureLockedSingleOutput,
    },
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::{rand_bytes, rand_bytes_array};

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn from_signature_locked_single() {
    let sls =
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 1).unwrap();
    let output = Output::from(sls.clone());

    assert_eq!(output.kind(), 0);
    assert!(matches!(output, Output::SignatureLockedSingle(output) if {output == sls}));
}

#[test]
fn from_signature_locked_asset() {
    let sla = SignatureLockedAssetOutput::new(
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        vec![AssetBalance::new(AssetId::new(rand_bytes_array()), 1000)],
    )
    .unwrap();
    let output = Output::from(sla.clone());

    assert_eq!(output.kind(), 1);
    assert!(matches!(output, Output::SignatureLockedAsset(output) if {output == sla}));
}

#[test]
fn packed_len() {
    let output = Output::from(
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 1).unwrap(),
    );

    assert_eq!(output.packed_len(), 1 + 1 + 32 + 8);
    assert_eq!(output.pack_to_vec().len(), 1 + 1 + 32 + 8);
}

#[test]
fn packable_round_trip() {
    let output_1 = Output::from(
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 1_000)
            .unwrap(),
    );
    let output_2 = Output::unpack_from_slice(output_1.pack_to_vec()).unwrap();

    assert_eq!(output_1, output_2);
}

#[test]
fn unpack_invalid_tag() {
    let mut bytes = vec![2, 0];
    bytes.extend(rand_bytes(32));
    bytes.extend(vec![128, 0, 0, 0, 0, 0, 0, 0]);

    let output = Output::unpack_from_slice(bytes);

    assert!(matches!(
        output,
        Err(UnpackError::Packable(MessageUnpackError::Output(
            OutputUnpackError::InvalidKind(2)
        ))),
    ));
}

#[test]
fn serde_round_trip() {
    let output_1 = Output::from(
        SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 1_000)
            .unwrap(),
    );
    let json = serde_json::to_string(&output_1).unwrap();
    let output_2 = serde_json::from_str::<Output>(&json).unwrap();

    assert_eq!(output_1, output_2);
}

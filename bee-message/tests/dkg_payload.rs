// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;

#[test]
fn kind() {
    assert_eq!(DkgPayload::KIND, 4);
}

#[test]
fn encrypted_deal_new() {
    let deal = EncryptedDeal::builder()
        .with_dh_key(vec![])
        .with_nonce(vec![])
        .with_encrypted_share(vec![])
        .with_threshold(10)
        .with_commitments(vec![])
        .finish();

    assert!(deal.is_ok());
}

#[test]
fn encrypted_deal_packed_len() {
    let deal = EncryptedDeal::builder()
        .with_dh_key(vec![])
        .with_nonce(vec![])
        .with_encrypted_share(vec![])
        .with_threshold(10)
        .with_commitments(vec![])
        .finish()
        .unwrap();

    assert_eq!(deal.packed_len(), 4 + 4 + 4 + 4 + 4);
}

#[test]
fn encryped_deal_unwrap() {
    let bytes = vec![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0];

    let deal = EncryptedDeal::unpack_from_slice(bytes);

    assert!(deal.is_ok());
}

#[test]
fn encrypted_deal_round_trip() {
    let deal_a = EncryptedDeal::builder()
        .with_dh_key(vec![])
        .with_nonce(vec![])
        .with_encrypted_share(vec![])
        .with_threshold(10)
        .with_commitments(vec![])
        .finish()
        .unwrap();

    let deal_b = EncryptedDeal::unpack_from_slice(deal_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(deal_a, deal_b);
}

#[test]
fn dkg_new() {
    let dkg = DkgPayload::builder()
        .with_version(0)
        .with_instance_id(1)
        .with_from_index(20)
        .with_to_index(32)
        .with_deal(
            EncryptedDeal::builder()
                .with_dh_key(vec![])
                .with_nonce(vec![])
                .with_encrypted_share(vec![])
                .with_threshold(10)
                .with_commitments(vec![])
                .finish()
                .unwrap(),
        )
        .finish();

    assert!(dkg.is_ok());
}

#[test]
fn dkg_packed_len() {
    let dkg = DkgPayload::builder()
        .with_version(0)
        .with_instance_id(1)
        .with_from_index(20)
        .with_to_index(32)
        .with_deal(
            EncryptedDeal::builder()
                .with_dh_key(vec![])
                .with_nonce(vec![])
                .with_encrypted_share(vec![])
                .with_threshold(10)
                .with_commitments(vec![])
                .finish()
                .unwrap(),
        )
        .finish()
        .unwrap();

    assert_eq!(dkg.packed_len(), 1 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4,);
}

#[test]
fn dkg_unpack_valid() {
    let bytes = vec![
        0u8, 0, 0, 0, 1, 0, 0, 0, 20, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0,
    ];

    let dkg = DkgPayload::unpack_from_slice(bytes);

    assert!(dkg.is_ok());
}

#[test]
fn dkg_round_trip() {
    let dkg_a = DkgPayload::builder()
        .with_version(0)
        .with_instance_id(1)
        .with_from_index(20)
        .with_to_index(32)
        .with_deal(
            EncryptedDeal::builder()
                .with_dh_key(vec![])
                .with_nonce(vec![])
                .with_encrypted_share(vec![])
                .with_threshold(10)
                .with_commitments(vec![])
                .finish()
                .unwrap(),
        )
        .finish()
        .unwrap();

    let dkg_b = DkgPayload::unpack_from_slice(dkg_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(dkg_a, dkg_b);
}

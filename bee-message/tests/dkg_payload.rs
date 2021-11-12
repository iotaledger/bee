// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::{
    drng::{DkgPayload, EncryptedDeal},
    MessagePayload,
};
use bee_packable::{Packable, PackableExt};
use bee_test::rand::bytes::rand_bytes;

#[test]
fn kind() {
    assert_eq!(DkgPayload::KIND, 4);
}

#[test]
fn version() {
    assert_eq!(DkgPayload::VERSION, 0);
}

#[test]
fn encrypted_deal_new() {
    let deal = EncryptedDeal::builder()
        .with_dh_key(rand_bytes(128))
        .with_nonce(rand_bytes(12))
        .with_encrypted_share(rand_bytes(128))
        .with_threshold(10)
        .with_commitments(rand_bytes(12))
        .finish();

    assert!(deal.is_ok());
}

#[test]
fn encrypted_deal_accessors_eq() {
    let dh_key = rand_bytes(128);
    let nonce = rand_bytes(12);
    let encrypted_share = rand_bytes(128);
    let commitments = rand_bytes(12);

    let deal = EncryptedDeal::builder()
        .with_dh_key(dh_key.clone())
        .with_nonce(nonce.clone())
        .with_encrypted_share(encrypted_share.clone())
        .with_threshold(10)
        .with_commitments(commitments.clone())
        .finish()
        .unwrap();

    assert_eq!(deal.dh_key(), dh_key);
    assert_eq!(deal.nonce(), nonce);
    assert_eq!(deal.encrypted_share(), encrypted_share);
    assert_eq!(deal.threshold(), 10);
    assert_eq!(deal.commitments(), commitments);
}

#[test]
fn encrypted_deal_packed_len() {
    let deal = EncryptedDeal::builder()
        .with_dh_key(rand_bytes(128))
        .with_nonce(rand_bytes(12))
        .with_encrypted_share(rand_bytes(128))
        .with_threshold(10)
        .with_commitments(rand_bytes(12))
        .finish()
        .unwrap();

    assert_eq!(deal.packed_len(), 4 + 4 + 4 + 4 + 4 + 128 + 12 + 128 + 12);
}

#[test]
fn encryped_deal_unwrap() {
    let bytes = vec![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0];

    let deal = EncryptedDeal::unpack_verified(bytes);

    assert!(deal.is_ok());
}

#[test]
fn encrypted_deal_packable_round_trip() {
    let deal_a = EncryptedDeal::builder()
        .with_dh_key(rand_bytes(128))
        .with_nonce(rand_bytes(12))
        .with_encrypted_share(rand_bytes(128))
        .with_threshold(10)
        .with_commitments(rand_bytes(12))
        .finish()
        .unwrap();

    let deal_b = EncryptedDeal::unpack_verified(deal_a.pack_to_vec()).unwrap();

    assert_eq!(deal_a, deal_b);
}

#[test]
fn dkg_new() {
    let dkg = DkgPayload::builder()
        .with_instance_id(1)
        .with_from_index(20)
        .with_to_index(32)
        .with_deal(
            EncryptedDeal::builder()
                .with_dh_key(rand_bytes(128))
                .with_nonce(rand_bytes(12))
                .with_encrypted_share(rand_bytes(128))
                .with_threshold(10)
                .with_commitments(rand_bytes(12))
                .finish()
                .unwrap(),
        )
        .finish();

    assert!(dkg.is_ok());
}

#[test]
fn dkg_unpack_valid() {
    let bytes = vec![
        0, 0, 0, 1, 0, 0, 0, 20, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0,
    ];

    let dkg = DkgPayload::unpack_verified(bytes);

    assert!(dkg.is_ok());
}

#[test]
fn accessors_eq() {
    let deal = EncryptedDeal::builder()
        .with_dh_key(rand_bytes(128))
        .with_nonce(rand_bytes(12))
        .with_encrypted_share(rand_bytes(128))
        .with_threshold(10)
        .with_commitments(rand_bytes(12))
        .finish()
        .unwrap();

    let dkg = DkgPayload::builder()
        .with_instance_id(1)
        .with_from_index(20)
        .with_to_index(32)
        .with_deal(deal.clone())
        .finish()
        .unwrap();

    assert_eq!(dkg.instance_id(), 1);
    assert_eq!(dkg.from_index(), 20);
    assert_eq!(dkg.to_index(), 32);
    assert_eq!(dkg.deal(), &deal);
}

#[test]
fn dkg_packed_len() {
    let dkg = DkgPayload::builder()
        .with_instance_id(1)
        .with_from_index(20)
        .with_to_index(32)
        .with_deal(
            EncryptedDeal::builder()
                .with_dh_key(rand_bytes(128))
                .with_nonce(rand_bytes(12))
                .with_encrypted_share(rand_bytes(128))
                .with_threshold(10)
                .with_commitments(rand_bytes(12))
                .finish()
                .unwrap(),
        )
        .finish()
        .unwrap();

    assert_eq!(dkg.packed_len(), 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 128 + 12 + 128 + 12);
}

#[test]
fn dkg_packable_round_trip() {
    let dkg_a = DkgPayload::builder()
        .with_instance_id(1)
        .with_from_index(20)
        .with_to_index(32)
        .with_deal(
            EncryptedDeal::builder()
                .with_dh_key(rand_bytes(128))
                .with_nonce(rand_bytes(12))
                .with_encrypted_share(rand_bytes(128))
                .with_threshold(10)
                .with_commitments(rand_bytes(12))
                .finish()
                .unwrap(),
        )
        .finish()
        .unwrap();

    let dkg_b = DkgPayload::unpack_verified(dkg_a.pack_to_vec()).unwrap();

    assert_eq!(dkg_a, dkg_b);
}

#[test]
fn serde_round_trip() {
    let dkg_payload_1 = DkgPayload::builder()
        .with_instance_id(1)
        .with_from_index(20)
        .with_to_index(32)
        .with_deal(
            EncryptedDeal::builder()
                .with_dh_key(rand_bytes(128))
                .with_nonce(rand_bytes(12))
                .with_encrypted_share(rand_bytes(128))
                .with_threshold(10)
                .with_commitments(rand_bytes(12))
                .finish()
                .unwrap(),
        )
        .finish()
        .unwrap();
    let json = serde_json::to_string(&dkg_payload_1).unwrap();
    let dkg_payload_2 = serde_json::from_str::<DkgPayload>(&json).unwrap();

    assert_eq!(dkg_payload_1, dkg_payload_2);
}

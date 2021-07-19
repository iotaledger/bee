// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;
use bee_test::rand::bytes::rand_bytes_array;

#[test]
fn kind() {
    assert_eq!(
        UnlockBlock::from(SignatureUnlock::from(Ed25519Signature::new(
            rand_bytes_array(),
            rand_bytes_array(),
        )))
        .kind(),
        0
    );
    assert_eq!(UnlockBlock::from(ReferenceUnlock::new(0).unwrap()).kind(), 1);
}

#[test]
fn new_invalid_first_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![ReferenceUnlock::new(42).unwrap().into()]),
        Err(ValidationError::InvalidUnlockBlockReference(0)),
    ));
}

#[test]
fn new_invalid_self_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Ed25519Signature::new([0; 32], [0; 64])).into(),
            ReferenceUnlock::new(1).unwrap().into()
        ]),
        Err(ValidationError::InvalidUnlockBlockReference(1)),
    ));
}

#[test]
fn new_invalid_future_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Ed25519Signature::new([0; 32], [0; 64])).into(),
            ReferenceUnlock::new(2).unwrap().into(),
            SignatureUnlock::from(Ed25519Signature::new([1; 32], [1; 64])).into(),
        ]),
        Err(ValidationError::InvalidUnlockBlockReference(1)),
    ));
}

#[test]
fn new_invalid_reference_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Ed25519Signature::new([0; 32], [0; 64])).into(),
            ReferenceUnlock::new(0).unwrap().into(),
            ReferenceUnlock::new(1).unwrap().into()
        ]),
        Err(ValidationError::InvalidUnlockBlockReference(2)),
    ));
}

#[test]
fn new_invalid_duplicate_signature() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Ed25519Signature::new([0; 32], [0; 64])).into(),
            ReferenceUnlock::new(0).unwrap().into(),
            ReferenceUnlock::new(0).unwrap().into(),
            SignatureUnlock::from(Ed25519Signature::new([1; 32], [1; 64])).into(),
            SignatureUnlock::from(Ed25519Signature::new([2; 32], [2; 64])).into(),
            SignatureUnlock::from(Ed25519Signature::new([2; 32], [2; 64])).into(),
            SignatureUnlock::from(Ed25519Signature::new([3; 32], [3; 64])).into(),
            ReferenceUnlock::new(3).unwrap().into()
        ]),
        Err(ValidationError::DuplicateSignature(5)),
    ));
}

#[test]
fn new_invalid_too_many_blocks() {
    assert!(matches!(
        UnlockBlocks::new(vec![ReferenceUnlock::new(0).unwrap().into(); 300]),
        Err(ValidationError::InvalidUnlockBlockCount(300)),
    ));
}

#[test]
fn new_valid() {
    assert!(UnlockBlocks::new(vec![
        SignatureUnlock::from(Ed25519Signature::new([0; 32], [0; 64])).into(),
        ReferenceUnlock::new(0).unwrap().into(),
        ReferenceUnlock::new(0).unwrap().into(),
        SignatureUnlock::from(Ed25519Signature::new([1; 32], [1; 64])).into(),
        SignatureUnlock::from(Ed25519Signature::new([2; 32], [2; 64])).into(),
        SignatureUnlock::from(Ed25519Signature::new([3; 32], [3; 64])).into(),
        ReferenceUnlock::new(3).unwrap().into(),
        ReferenceUnlock::new(4).unwrap().into(),
        ReferenceUnlock::new(3).unwrap().into(),
        ReferenceUnlock::new(4).unwrap().into(),
        ReferenceUnlock::new(5).unwrap().into(),
        SignatureUnlock::from(Ed25519Signature::new([4; 32], [4; 64])).into(),
        ReferenceUnlock::new(11).unwrap().into(),
        SignatureUnlock::from(Ed25519Signature::new([5; 32], [5; 64])).into(),
    ])
    .is_ok());
}

#[test]
fn get_none() {
    assert!(UnlockBlocks::new(vec![
        SignatureUnlock::from(Ed25519Signature::new([0; 32], [0; 64])).into(),
    ])
    .unwrap()
    .get(42)
    .is_none());
}

#[test]
fn get_signature() {
    let signature = UnlockBlock::from(SignatureUnlock::from(Ed25519Signature::new([0; 32], [0; 64])));

    assert_eq!(
        UnlockBlocks::new(vec![signature.clone()]).unwrap().get(0),
        Some(&signature)
    );
}

#[test]
fn get_signature_through_reference() {
    let signature = UnlockBlock::from(SignatureUnlock::from(Ed25519Signature::new([0; 32], [0; 64])));

    assert_eq!(
        UnlockBlocks::new(vec![signature.clone(), ReferenceUnlock::new(0).unwrap().into()])
            .unwrap()
            .get(1),
        Some(&signature)
    );
}

#[test]
fn packable_round_trip() {
    let blocks_a = UnlockBlocks::new(vec![
        SignatureUnlock::from(Ed25519Signature::new([0; 32], [0; 64])).into(),
        ReferenceUnlock::new(0).unwrap().into(),
        ReferenceUnlock::new(0).unwrap().into(),
        SignatureUnlock::from(Ed25519Signature::new([1; 32], [1; 64])).into(),
        SignatureUnlock::from(Ed25519Signature::new([2; 32], [2; 64])).into(),
        SignatureUnlock::from(Ed25519Signature::new([3; 32], [3; 64])).into(),
        ReferenceUnlock::new(3).unwrap().into(),
        ReferenceUnlock::new(4).unwrap().into(),
    ])
    .unwrap();

    let blocks_b = UnlockBlocks::unpack_from_slice(blocks_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(blocks_a, blocks_b);
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_test::rand::bytes::{rand_bytes, rand_bytes_array};

#[test]
fn kind() {
    assert_eq!(
        UnlockBlock::from(SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new(
            rand_bytes_array(),
            rand_bytes(64).try_into().unwrap(),
        ))))
        .kind(),
        0
    );
    assert_eq!(UnlockBlock::from(ReferenceUnlockBlock::new(0).unwrap()).kind(), 1);
}

#[test]
fn new_invalid_first_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![ReferenceUnlockBlock::new(42).unwrap().into()]),
        Err(Error::InvalidUnlockBlockReference(0)),
    ));
}

#[test]
fn new_invalid_self_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlockBlock::new(1).unwrap().into()
        ]),
        Err(Error::InvalidUnlockBlockReference(1)),
    ));
}

#[test]
fn new_invalid_future_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlockBlock::new(2).unwrap().into(),
            SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([1; 32], [1; 64]))).into(),
        ]),
        Err(Error::InvalidUnlockBlockReference(1)),
    ));
}

#[test]
fn new_invalid_reference_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlockBlock::new(0).unwrap().into(),
            ReferenceUnlockBlock::new(1).unwrap().into()
        ]),
        Err(Error::InvalidUnlockBlockReference(2)),
    ));
}

#[test]
fn new_invalid_duplicate_signature() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlockBlock::new(0).unwrap().into(),
            ReferenceUnlockBlock::new(0).unwrap().into(),
            SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([1; 32], [1; 64]))).into(),
            SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([2; 32], [2; 64]))).into(),
            SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([2; 32], [2; 64]))).into(),
            SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([3; 32], [3; 64]))).into(),
            ReferenceUnlockBlock::new(3).unwrap().into()
        ]),
        Err(Error::DuplicateSignatureUnlockBlock(5)),
    ));
}

#[test]
fn new_invalid_too_many_blocks() {
    assert!(matches!(
        UnlockBlocks::new(vec![ReferenceUnlockBlock::new(0).unwrap().into(); 300]),
        Err(Error::InvalidUnlockBlockCount(300)),
    ));
}

#[test]
fn new_valid() {
    assert!(UnlockBlocks::new(vec![
        SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
        ReferenceUnlockBlock::new(0).unwrap().into(),
        ReferenceUnlockBlock::new(0).unwrap().into(),
        SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([1; 32], [1; 64]))).into(),
        SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([2; 32], [2; 64]))).into(),
        SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([3; 32], [3; 64]))).into(),
        ReferenceUnlockBlock::new(3).unwrap().into(),
        ReferenceUnlockBlock::new(4).unwrap().into(),
        ReferenceUnlockBlock::new(3).unwrap().into(),
        ReferenceUnlockBlock::new(4).unwrap().into(),
        ReferenceUnlockBlock::new(5).unwrap().into(),
        SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([4; 32], [4; 64]))).into(),
        ReferenceUnlockBlock::new(11).unwrap().into(),
        SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new([5; 32], [5; 64]))).into(),
    ])
    .is_ok());
}

#[test]
fn get_none() {
    assert!(UnlockBlocks::new(vec![SignatureUnlockBlock::from(Signature::from(
        Ed25519Signature::new([0; 32], [0; 64])
    ))
    .into()])
    .unwrap()
    .get(42)
    .is_none());
}

#[test]
fn get_signature() {
    let signature = UnlockBlock::from(SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new(
        [0; 32], [0; 64],
    ))));

    assert_eq!(
        UnlockBlocks::new(vec![signature.clone()]).unwrap().get(0),
        Some(&signature)
    );
}

#[test]
fn get_signature_through_reference() {
    let signature = UnlockBlock::from(SignatureUnlockBlock::from(Signature::from(Ed25519Signature::new(
        [0; 32], [0; 64],
    ))));

    assert_eq!(
        UnlockBlocks::new(vec![signature.clone(), ReferenceUnlockBlock::new(0).unwrap().into()])
            .unwrap()
            .get(1),
        Some(&signature)
    );
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;

#[test]
fn new_invalid_first_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![ReferenceUnlock::new(42).unwrap().into()]),
        Err(Error::InvalidUnlockBlockReference(0)),
    ));
}

#[test]
fn new_invalid_self_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Ed25519Signature::new([0; 32], Box::new([0; 64]))).into(),
            ReferenceUnlock::new(1).unwrap().into()
        ]),
        Err(Error::InvalidUnlockBlockReference(1)),
    ));
}

#[test]
fn new_invalid_future_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Ed25519Signature::new([0; 32], Box::new([0; 64]))).into(),
            ReferenceUnlock::new(2).unwrap().into(),
            SignatureUnlock::from(Ed25519Signature::new([1; 32], Box::new([1; 64]))).into(),
        ]),
        Err(Error::InvalidUnlockBlockReference(1)),
    ));
}

#[test]
fn new_invalid_reference_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Ed25519Signature::new([0; 32], Box::new([0; 64]))).into(),
            ReferenceUnlock::new(0).unwrap().into(),
            ReferenceUnlock::new(1).unwrap().into()
        ]),
        Err(Error::InvalidUnlockBlockReference(2)),
    ));
}

#[test]
fn new_invalid_duplicate_signature() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Ed25519Signature::new([0; 32], Box::new([0; 64]))).into(),
            ReferenceUnlock::new(0).unwrap().into(),
            ReferenceUnlock::new(0).unwrap().into(),
            SignatureUnlock::from(Ed25519Signature::new([1; 32], Box::new([1; 64]))).into(),
            SignatureUnlock::from(Ed25519Signature::new([2; 32], Box::new([2; 64]))).into(),
            SignatureUnlock::from(Ed25519Signature::new([2; 32], Box::new([2; 64]))).into(),
            SignatureUnlock::from(Ed25519Signature::new([3; 32], Box::new([3; 64]))).into(),
            ReferenceUnlock::new(3).unwrap().into()
        ]),
        Err(Error::DuplicateSignature(5)),
    ));
}

#[test]
fn new_valid() {
    assert!(UnlockBlocks::new(vec![
        SignatureUnlock::from(Ed25519Signature::new([0; 32], Box::new([0; 64]))).into(),
        ReferenceUnlock::new(0).unwrap().into(),
        ReferenceUnlock::new(0).unwrap().into(),
        SignatureUnlock::from(Ed25519Signature::new([1; 32], Box::new([1; 64]))).into(),
        SignatureUnlock::from(Ed25519Signature::new([2; 32], Box::new([2; 64]))).into(),
        SignatureUnlock::from(Ed25519Signature::new([3; 32], Box::new([3; 64]))).into(),
        ReferenceUnlock::new(3).unwrap().into(),
        ReferenceUnlock::new(4).unwrap().into(),
        ReferenceUnlock::new(3).unwrap().into(),
        ReferenceUnlock::new(4).unwrap().into(),
        ReferenceUnlock::new(5).unwrap().into(),
        SignatureUnlock::from(Ed25519Signature::new([4; 32], Box::new([4; 64]))).into(),
        ReferenceUnlock::new(11).unwrap().into(),
        SignatureUnlock::from(Ed25519Signature::new([5; 32], Box::new([5; 64]))).into(),
    ])
    .is_ok());
}

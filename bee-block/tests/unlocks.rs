// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    rand::bytes::{rand_bytes, rand_bytes_array},
    signature::{Ed25519Signature, Signature},
    unlock::{ReferenceUnlock, SignatureUnlock, Unlock, Unlocks},
    Error,
};
use packable::bounded::TryIntoBoundedU16Error;

#[test]
fn kind() {
    assert_eq!(
        Unlock::from(SignatureUnlock::from(Signature::from(Ed25519Signature::new(
            rand_bytes_array(),
            rand_bytes(64).try_into().unwrap(),
        ))))
        .kind(),
        0
    );
    assert_eq!(Unlock::from(ReferenceUnlock::new(0).unwrap()).kind(), 1);
}

#[test]
fn new_invalid_first_reference() {
    assert!(matches!(
        Unlocks::new(vec![ReferenceUnlock::new(42).unwrap().into()]),
        Err(Error::InvalidUnlockReference(0)),
    ));
}

#[test]
fn new_invalid_self_reference() {
    assert!(matches!(
        Unlocks::new(vec![
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlock::new(1).unwrap().into()
        ]),
        Err(Error::InvalidUnlockReference(1)),
    ));
}

#[test]
fn new_invalid_future_reference() {
    assert!(matches!(
        Unlocks::new(vec![
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlock::new(2).unwrap().into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([1; 32], [1; 64]))).into(),
        ]),
        Err(Error::InvalidUnlockReference(1)),
    ));
}

#[test]
fn new_invalid_reference_reference() {
    assert!(matches!(
        Unlocks::new(vec![
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlock::new(0).unwrap().into(),
            ReferenceUnlock::new(1).unwrap().into()
        ]),
        Err(Error::InvalidUnlockReference(2)),
    ));
}

#[test]
fn new_invalid_duplicate_signature() {
    assert!(matches!(
        Unlocks::new(vec![
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlock::new(0).unwrap().into(),
            ReferenceUnlock::new(0).unwrap().into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([1; 32], [1; 64]))).into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([2; 32], [2; 64]))).into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([2; 32], [2; 64]))).into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([3; 32], [3; 64]))).into(),
            ReferenceUnlock::new(3).unwrap().into()
        ]),
        Err(Error::DuplicateSignatureUnlock(5)),
    ));
}

#[test]
fn new_invalid_too_many_blocks() {
    assert!(matches!(
        Unlocks::new(vec![ReferenceUnlock::new(0).unwrap().into(); 300]),
        Err(Error::InvalidUnlockCount(TryIntoBoundedU16Error::Invalid(300))),
    ));
}

#[test]
fn new_valid() {
    assert!(
        Unlocks::new(vec![
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlock::new(0).unwrap().into(),
            ReferenceUnlock::new(0).unwrap().into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([1; 32], [1; 64]))).into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([2; 32], [2; 64]))).into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([3; 32], [3; 64]))).into(),
            ReferenceUnlock::new(3).unwrap().into(),
            ReferenceUnlock::new(4).unwrap().into(),
            ReferenceUnlock::new(3).unwrap().into(),
            ReferenceUnlock::new(4).unwrap().into(),
            ReferenceUnlock::new(5).unwrap().into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([4; 32], [4; 64]))).into(),
            ReferenceUnlock::new(11).unwrap().into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([5; 32], [5; 64]))).into(),
        ])
        .is_ok()
    );
}

#[test]
fn get_none() {
    assert!(
        Unlocks::new(vec![
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into()
        ])
        .unwrap()
        .get(42)
        .is_none()
    );
}

#[test]
fn get_signature() {
    let signature = Unlock::from(SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        [0; 32], [0; 64],
    ))));

    assert_eq!(Unlocks::new(vec![signature.clone()]).unwrap().get(0), Some(&signature));
}

#[test]
fn get_signature_through_reference() {
    let signature = Unlock::from(SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        [0; 32], [0; 64],
    ))));

    assert_eq!(
        Unlocks::new(vec![signature.clone(), ReferenceUnlock::new(0).unwrap().into()])
            .unwrap()
            .get(1),
        Some(&signature)
    );
}

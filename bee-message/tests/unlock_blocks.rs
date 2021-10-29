// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::{MessageUnpackError, ValidationError},
    signature::{Ed25519Signature, Signature},
    unlock::{ReferenceUnlock, SignatureUnlock, UnlockBlock, UnlockBlockUnpackError, UnlockBlocks},
};
use bee_packable::{packable::VecPrefixLengthError, InvalidBoundedU16, Packable, UnpackError};
use bee_test::rand::bytes::{rand_bytes, rand_bytes_array};

#[test]
fn kind() {
    assert_eq!(
        UnlockBlock::from(SignatureUnlock::from(Signature::from(Ed25519Signature::new(
            rand_bytes_array(),
            rand_bytes_array(),
        ))))
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
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlock::new(1).unwrap().into()
        ]),
        Err(ValidationError::InvalidUnlockBlockReference(1)),
    ));
}

#[test]
fn new_invalid_future_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlock::new(2).unwrap().into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([1; 32], [1; 64]))).into(),
        ]),
        Err(ValidationError::InvalidUnlockBlockReference(1)),
    ));
}

#[test]
fn new_invalid_reference_reference() {
    assert!(matches!(
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
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
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
            ReferenceUnlock::new(0).unwrap().into(),
            ReferenceUnlock::new(0).unwrap().into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([1; 32], [1; 64]))).into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([2; 32], [2; 64]))).into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([2; 32], [2; 64]))).into(),
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([3; 32], [3; 64]))).into(),
            ReferenceUnlock::new(3).unwrap().into()
        ]),
        Err(ValidationError::DuplicateSignature(5)),
    ));
}

#[test]
fn new_invalid_too_many_blocks() {
    assert!(matches!(
        UnlockBlocks::new(vec![ReferenceUnlock::new(0).unwrap().into(); 300]),
        Err(ValidationError::InvalidUnlockBlockCount(VecPrefixLengthError::Invalid(
            InvalidBoundedU16(300)
        ))),
    ));
}

#[test]
fn new_valid() {
    assert!(
        UnlockBlocks::new(vec![
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
        UnlockBlocks::new(vec![
            SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
        ])
        .unwrap()
        .get(42)
        .is_none()
    );
}

#[test]
fn get_signature() {
    let signature = UnlockBlock::from(SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        [0; 32], [0; 64],
    ))));

    assert_eq!(
        UnlockBlocks::new(vec![signature.clone()]).unwrap().get(0),
        Some(&signature)
    );
}

#[test]
fn get_signature_through_reference() {
    let signature = UnlockBlock::from(SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        [0; 32], [0; 64],
    ))));

    assert_eq!(
        UnlockBlocks::new(vec![signature.clone(), ReferenceUnlock::new(0).unwrap().into()])
            .unwrap()
            .get(1),
        Some(&signature)
    );
}

#[test]
fn unpack_invalid_unlock_block_kind() {
    let mut bytes = vec![1, 0];
    bytes.extend([2, 0]);
    bytes.extend(rand_bytes(32));
    bytes.extend(rand_bytes(64));

    let unlock_blocks = UnlockBlocks::unpack_from_slice(bytes);

    assert!(matches!(
        unlock_blocks,
        Err(UnpackError::Packable(MessageUnpackError::UnlockBlock(
            UnlockBlockUnpackError::InvalidKind(2)
        ))),
    ));
}

#[test]
fn packable_round_trip() {
    let blocks_a = UnlockBlocks::new(vec![
        SignatureUnlock::from(Signature::from(Ed25519Signature::new([0; 32], [0; 64]))).into(),
        ReferenceUnlock::new(0).unwrap().into(),
        ReferenceUnlock::new(0).unwrap().into(),
        SignatureUnlock::from(Signature::from(Ed25519Signature::new([1; 32], [1; 64]))).into(),
        SignatureUnlock::from(Signature::from(Ed25519Signature::new([2; 32], [2; 64]))).into(),
        SignatureUnlock::from(Signature::from(Ed25519Signature::new([3; 32], [3; 64]))).into(),
        ReferenceUnlock::new(3).unwrap().into(),
        ReferenceUnlock::new(4).unwrap().into(),
    ])
    .unwrap();

    let blocks_b = UnlockBlocks::unpack_from_slice(blocks_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(blocks_a, blocks_b);
}

#[test]
fn serde_round_trip() {
    let unlock_blocks_1 = UnlockBlocks::new(vec![
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
    .unwrap();
    let json = serde_json::to_string(&unlock_blocks_1).unwrap();
    let unlock_blocks_2 = serde_json::from_str::<UnlockBlocks>(&json).unwrap();

    assert_eq!(unlock_blocks_1, unlock_blocks_2);
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    error::MessageUnpackError,
    signature::{Ed25519Signature, Signature},
    unlock::{ReferenceUnlock, SignatureUnlock, UnlockBlock, UnlockBlockUnpackError},
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::bytes::{rand_bytes, rand_bytes_array};

#[test]
fn from_signature() {
    let signature = SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        rand_bytes_array(),
        rand_bytes_array(),
    )));
    let unlock = UnlockBlock::from(signature.clone());

    assert_eq!(unlock.kind(), 0);
    assert!(matches!(unlock, UnlockBlock::Signature(unlock) if {unlock == signature}));
}

#[test]
fn from_reference() {
    let reference = ReferenceUnlock::new(42).unwrap();
    let unlock = UnlockBlock::from(reference.clone());

    assert_eq!(unlock.kind(), 1);
    assert!(matches!(unlock, UnlockBlock::Reference(unlock) if {unlock == reference}));
}

#[test]
fn packed_len() {
    let unlock = UnlockBlock::from(SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        rand_bytes_array(),
        rand_bytes_array(),
    ))));

    assert_eq!(unlock.packed_len(), 1 + 1 + 32 + 64);
    assert_eq!(unlock.pack_to_vec().unwrap().len(), 1 + 1 + 32 + 64);
}

#[test]
fn packable_round_trip() {
    let unlock_1 = UnlockBlock::from(SignatureUnlock::from(Signature::from(Ed25519Signature::new(
        rand_bytes_array(),
        rand_bytes_array(),
    ))));
    let unlock_2 = UnlockBlock::unpack_from_slice(unlock_1.pack_to_vec().unwrap()).unwrap();

    assert_eq!(unlock_1, unlock_2);
}

#[test]
fn unpack_invalid_tag() {
    let mut bytes = vec![2, 0];
    bytes.extend(rand_bytes(32));
    bytes.extend(rand_bytes(64));

    let unlock_block = UnlockBlock::unpack_from_slice(bytes);

    assert!(matches!(
        unlock_block,
        Err(UnpackError::Packable(MessageUnpackError::UnlockBlock(
            UnlockBlockUnpackError::InvalidKind(2)
        ))),
    ));
}

#[test]
fn serde_round_trip() {
    let unlock_block_1 = UnlockBlock::from(ReferenceUnlock::new(42).unwrap());
    let json = serde_json::to_string(&unlock_block_1).unwrap();
    let unlock_block_2 = serde_json::from_str::<UnlockBlock>(&json).unwrap();

    assert_eq!(unlock_block_1, unlock_block_2);
}

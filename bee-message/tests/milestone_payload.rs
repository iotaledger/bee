// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    milestone::MilestoneIndex,
    parent::Parents,
    payload::milestone::{MilestoneEssence, MilestoneOptions, MilestonePayload},
    signature::{Ed25519Signature, Signature},
    Error,
};
use bee_test::rand::{self, message::rand_message_ids, milestone::rand_milestone_id, parents::rand_parents};
use packable::{bounded::TryIntoBoundedU8Error, PackableExt};

#[test]
fn kind() {
    assert_eq!(MilestonePayload::KIND, 7);
}

#[test]
fn new_valid() {
    assert!(MilestonePayload::new(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_milestone_id(),
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![],
            MilestoneOptions::new(vec![]).unwrap(),
        )
        .unwrap(),
        vec![Signature::from(Ed25519Signature::new([0; 32], [0; 64]))]
    )
    .is_ok());
}

#[test]
fn new_invalid_no_signature() {
    assert!(matches!(
        MilestonePayload::new(
            MilestoneEssence::new(
                MilestoneIndex(0),
                0,
                rand_milestone_id(),
                rand_parents(),
                [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
                [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
                0,
                0,
                vec![],
                MilestoneOptions::new(vec![]).unwrap(),
            )
            .unwrap(),
            vec![]
        ),
        Err(Error::MilestoneInvalidSignatureCount(TryIntoBoundedU8Error::Invalid(0)))
    ));
}

#[test]
fn new_invalid_too_many_signatures() {
    assert!(matches!(
        MilestonePayload::new(
            MilestoneEssence::new(
                MilestoneIndex(0),
                0,
                rand_milestone_id(),
                rand_parents(),
                [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
                [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
                0,
                0,
                vec![],
                MilestoneOptions::new(vec![]).unwrap(),
            )
            .unwrap(),
            vec![Signature::from(Ed25519Signature::new([0; 32], [0; 64])); 300]
        ),
        Err(Error::MilestoneInvalidSignatureCount(TryIntoBoundedU8Error::Truncated(
            300
        )))
    ));
}

#[test]
fn packed_len() {
    let ms = MilestonePayload::new(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_milestone_id(),
            Parents::new(rand_message_ids(4)).unwrap(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![0x2a, 0x2a, 0x2a, 0x2a, 0x2a],
            MilestoneOptions::new(vec![]).unwrap(),
        )
        .unwrap(),
        vec![
            Signature::from(Ed25519Signature::new([0; 32], [0; 64])),
            Signature::from(Ed25519Signature::new([1; 32], [1; 64])),
        ],
    )
    .unwrap();

    assert_eq!(ms.packed_len(), 447);
    assert_eq!(ms.pack_to_vec().len(), 447);
}

#[test]
fn pack_unpack_valid() {
    let payload = MilestonePayload::new(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_milestone_id(),
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![],
            MilestoneOptions::new(vec![]).unwrap(),
        )
        .unwrap(),
        vec![Signature::from(Ed25519Signature::new([0; 32], [0; 64]))],
    )
    .unwrap();

    let packed = payload.pack_to_vec();

    assert_eq!(payload.packed_len(), packed.len());
    assert_eq!(payload, PackableExt::unpack_verified(&mut packed.as_slice()).unwrap())
}

#[test]
fn getters() {
    let essence = MilestoneEssence::new(
        rand::milestone::rand_milestone_index(),
        rand::number::rand_number::<u32>(),
        rand_milestone_id(),
        rand_parents(),
        [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
        [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
        0,
        0,
        vec![],
        MilestoneOptions::new(vec![]).unwrap(),
    )
    .unwrap();
    let signatures = vec![Signature::from(Ed25519Signature::new([0; 32], [0; 64]))];
    let milestone = MilestonePayload::new(essence.clone(), signatures.clone()).unwrap();

    assert_eq!(essence, *milestone.essence());

    assert_eq!(signatures.len(), milestone.signatures().len());
    for (s1, s2) in signatures.iter().zip(milestone.signatures()) {
        assert_eq!(s1, s2);
    }
}

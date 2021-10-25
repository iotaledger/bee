// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::PackableExt;
use bee_test::rand::{self, message::rand_message_ids, parents::rand_parents};

#[test]
fn kind() {
    assert_eq!(MilestonePayload::KIND, 1);
}

#[test]
fn new_valid() {
    assert!(
        MilestonePayload::new(
            MilestonePayloadEssence::new(
                MilestoneIndex(0),
                0,
                rand_parents(),
                [0; MILESTONE_MERKLE_PROOF_LENGTH],
                0,
                0,
                vec![[0; 32]],
                None,
            )
            .unwrap(),
            vec![[0; 64]],
        )
        .is_ok()
    );
}

#[test]
fn new_invalid_no_signature() {
    assert!(matches!(
        MilestonePayload::new(
            MilestonePayloadEssence::new(
                MilestoneIndex(0),
                0,
                rand_parents(),
                [0; MILESTONE_MERKLE_PROOF_LENGTH],
                0,
                0,
                vec![[0; 32]],
                None,
            )
            .unwrap(),
            vec![]
        ),
        Err(Error::MilestoneInvalidSignatureCount(0))
    ));
}

#[test]
fn new_invalid_too_many_signatures() {
    assert!(matches!(
        MilestonePayload::new(
            MilestonePayloadEssence::new(
                MilestoneIndex(0),
                0,
                rand_parents(),
                [0; MILESTONE_MERKLE_PROOF_LENGTH],
                0,
                0,
                vec![[0; 32]],
                None,
            )
            .unwrap(),
            vec![[0u8; 64]; 300],
        ),
        Err(Error::MilestoneInvalidSignatureCount(300))
    ));
}

#[test]
fn new_invalid_public_keys_sgnatures_count_mismatch() {
    assert!(matches!(
        MilestonePayload::new(
            MilestonePayloadEssence::new(
                MilestoneIndex(0),
                0,
                rand_parents(),
                [0; MILESTONE_MERKLE_PROOF_LENGTH],
                0,
                0,
                vec![[0; 32], [1; 32]],
                None,
            )
            .unwrap(),
            vec![[0; 64], [1; 64], [3; 64]],
        ),
        Err(Error::MilestonePublicKeysSignaturesCountMismatch(2, 3))
    ));
}

#[test]
fn packed_len() {
    let ms = MilestonePayload::new(
        MilestonePayloadEssence::new(
            MilestoneIndex(0),
            0,
            Parents::new(rand_message_ids(4)).unwrap(),
            [0; MILESTONE_MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![[0; 32], [1; 32]],
            None,
        )
        .unwrap(),
        vec![[0; 64], [1; 64]],
    )
    .unwrap();

    assert_eq!(ms.packed_len(), 379);
    assert_eq!(ms.pack_to_vec().unwrap().len(), 379);
}

#[test]
fn pack_unpack_valid() {
    let payload = MilestonePayload::new(
        MilestonePayloadEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MILESTONE_MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![[0; 32]],
            None,
        )
        .unwrap(),
        vec![[0; 64]],
    )
    .unwrap();

    let packed = payload.pack_to_vec().unwrap();

    assert_eq!(payload.packed_len(), packed.len());
    assert_eq!(payload, PackableExt::unpack_verified(&mut packed.as_slice()).unwrap())
}

#[test]
fn getters() {
    let essence = MilestonePayloadEssence::new(
        rand::milestone::rand_milestone_index(),
        rand::number::rand_number::<u64>(),
        rand_parents(),
        [0; MILESTONE_MERKLE_PROOF_LENGTH],
        0,
        0,
        vec![[0; 32]],
        None,
    )
    .unwrap();
    let signatures = vec![[0; 64]];
    let milestone = MilestonePayload::new(essence.clone(), signatures.clone()).unwrap();

    assert_eq!(essence, *milestone.essence());
    assert_eq!(signatures.len(), milestone.signatures().len());
    for (expected, found) in signatures.iter().zip(milestone.signatures()) {
        assert_eq!(expected, &**found);
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_test::rand::parents::rand_parents;

#[test]
fn kind() {
    assert_eq!(MilestonePayload::KIND, 1);
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
            vec![Box::new([0u8; 64]); 300],
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
            vec![Box::new([0; 64]), Box::new([1; 64]), Box::new([3; 64])],
        ),
        Err(Error::MilestonePublicKeysSignaturesCountMismatch(2, 3))
    ));
}

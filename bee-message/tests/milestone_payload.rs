// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_test::rand::parents::rand_parents;

#[test]
fn new_invalid_no_public_key() {
    assert!(matches!(
        MilestonePayloadEssence::new(0, 0, rand_parents(), [0; MILESTONE_MERKLE_PROOF_LENGTH], vec![], None,),
        Err(Error::MilestoneNoPublicKey)
    ));
}

#[test]
fn new_valid_sorted_unique_public_keys() {
    assert!(MilestonePayloadEssence::new(
        0,
        0,
        rand_parents(),
        [0; MILESTONE_MERKLE_PROOF_LENGTH],
        vec![[0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32]],
        None,
    )
    .is_ok());
}

#[test]
fn new_invalid_sorted_not_unique_public_keys() {
    assert!(matches!(
        MilestonePayloadEssence::new(
            0,
            0,
            rand_parents(),
            [0; MILESTONE_MERKLE_PROOF_LENGTH],
            vec![[0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [4; 32], [6; 32], [7; 32], [8; 32], [9; 32]],
            None,
        ),
        Err(Error::MilestonePublicKeysNotUniqueSorted)
    ));
}

#[test]
fn new_invalid_not_sorted_unique_public_keys() {
    assert!(matches!(
        MilestonePayloadEssence::new(
            0,
            0,
            rand_parents(),
            [0; MILESTONE_MERKLE_PROOF_LENGTH],
            vec![[0; 32], [1; 32], [3; 32], [2; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32]],
            None,
        ),
        Err(Error::MilestonePublicKeysNotUniqueSorted)
    ));
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_test::rand::parents::rand_parents;

#[test]
fn new_invalid_no_signature() {
    assert!(matches!(
        MilestonePayload::new(
            MilestonePayloadEssence::new(
                0,
                0,
                rand_parents(),
                [0; MILESTONE_MERKLE_PROOF_LENGTH],
                vec![[0; 32]],
                None,
            )
            .unwrap(),
            vec![]
        ),
        Err(Error::MilestoneNoSignature)
    ));
}

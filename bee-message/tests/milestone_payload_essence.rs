// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    input::TreasuryInput,
    milestone::MilestoneIndex,
    output::TreasuryOutput,
    payload::{
        milestone::{MilestoneEssence, MilestoneOption, MilestoneOptions, PowMilestoneOption, ReceiptMilestoneOption},
        TreasuryTransactionPayload,
    },
};
use bee_test::rand::{self, milestone::rand_milestone_id, parents::rand_parents};
use packable::PackableExt;

// TODO put back when the TIP is finished
// #[test]
// fn new_invalid_pow_score_non_zero() {
//     assert!(matches!(
//         MilestoneEssence::new(
//             MilestoneIndex(0),
//             0,
//             rand_milestone_id(),
//             rand_parents(),
//             [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
//             [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
//             0,
//             4242,
//             vec![],
//             MilestoneOptions::new(vec![]).unwrap(),
//         ),
//         Err(Error::InvalidPowScoreValues { nps: 0, npsmi: 4242 })
//     ));
// }

// TODO put back when the TIP is finished
// #[test]
// fn new_invalid_pow_score_lower_than_index() {
//     assert!(matches!(
//         MilestoneEssence::new(
//             MilestoneIndex(4242),
//             0,
//             rand_milestone_id(),
//             rand_parents(),
//             [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
//             [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
//             4000,
//             4241,
//             vec![],
//             MilestoneOptions::new(vec![]).unwrap(),
//         ),
//         Err(Error::InvalidPowScoreValues { nps: 4000, npsmi: 4241 })
//     ));
// }

#[test]
fn new_valid() {
    assert!(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_milestone_id(),
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            vec![],
            MilestoneOptions::new(vec![]).unwrap(),
        )
        .is_ok()
    );
}

#[test]
fn getters() {
    let index = rand::milestone::rand_milestone_index();
    let timestamp = rand::number::rand_number::<u32>();
    let previous_milestone_id = rand_milestone_id();
    let parents = rand_parents();
    let confirmed_merkle_proof = [0; MilestoneEssence::MERKLE_PROOF_LENGTH];
    let applied_merkle_proof = [0; MilestoneEssence::MERKLE_PROOF_LENGTH];
    let next_pow_score = 0;
    let next_pow_score_milestone_index = 0;
    let receipt = ReceiptMilestoneOption::new(
        index,
        true,
        vec![rand::receipt::rand_migrated_funds_entry()],
        TreasuryTransactionPayload::new(
            TreasuryInput::new(rand::milestone::rand_milestone_id()),
            TreasuryOutput::new(1_000_000).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let options = MilestoneOptions::new(vec![
        MilestoneOption::Receipt(receipt.clone()),
        MilestoneOption::Pow(PowMilestoneOption::new(next_pow_score, next_pow_score_milestone_index).unwrap()),
    ])
    .unwrap();

    let milestone_payload = MilestoneEssence::new(
        index,
        timestamp,
        previous_milestone_id,
        parents.clone(),
        confirmed_merkle_proof,
        applied_merkle_proof,
        vec![],
        options,
    )
    .unwrap();

    assert_eq!(milestone_payload.index(), index);
    assert_eq!(milestone_payload.timestamp(), timestamp);
    assert_eq!(milestone_payload.previous_milestone_id(), &previous_milestone_id);
    assert_eq!(*milestone_payload.parents(), parents);
    assert_eq!(milestone_payload.confirmed_merkle_proof(), confirmed_merkle_proof);
    assert_eq!(milestone_payload.applied_merkle_proof(), applied_merkle_proof);
    assert_eq!(
        milestone_payload.options().pow().unwrap().next_pow_score(),
        next_pow_score
    );
    assert_eq!(
        milestone_payload
            .options()
            .pow()
            .unwrap()
            .next_pow_score_milestone_index(),
        next_pow_score_milestone_index
    );
    assert_eq!(*milestone_payload.options().receipt().unwrap(), receipt);
}

#[test]
fn pack_unpack_valid() {
    let milestone_payload = MilestoneEssence::new(
        MilestoneIndex(0),
        0,
        rand_milestone_id(),
        rand_parents(),
        [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
        [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
        vec![],
        MilestoneOptions::new(vec![]).unwrap(),
    )
    .unwrap();

    let packed = milestone_payload.pack_to_vec();

    assert_eq!(
        MilestoneEssence::unpack_verified(&mut packed.as_slice()).unwrap(),
        milestone_payload,
    );
}

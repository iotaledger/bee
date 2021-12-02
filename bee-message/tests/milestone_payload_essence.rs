// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::{
    input::{Input, TreasuryInput},
    milestone::MilestoneIndex,
    output::{Output, TreasuryOutput},
    payload::{
        milestone::MilestonePayloadEssence, IndexationPayload, Payload, ReceiptPayload, TreasuryTransactionPayload,
    },
    Error,
};
use bee_test::rand::{self, bytes::rand_bytes_array, parents::rand_parents};

#[test]
fn new_invalid_pow_score_non_zero() {
    assert!(matches!(
        MilestonePayloadEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH],
            0,
            4242,
            vec![],
            None,
        ),
        Err(Error::InvalidPowScoreValues(0, 4242))
    ));
}

#[test]
fn new_invalid_pow_score_lower_than_index() {
    assert!(matches!(
        MilestonePayloadEssence::new(
            MilestoneIndex(4242),
            0,
            rand_parents(),
            [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH],
            4000,
            4241,
            vec![],
            None,
        ),
        Err(Error::InvalidPowScoreValues(4000, 4241))
    ));
}

#[test]
fn new_invalid_no_public_key() {
    assert!(matches!(
        MilestonePayloadEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![],
            None,
        ),
        Err(Error::MilestoneInvalidPublicKeyCount(0))
    ));
}

#[test]
fn new_invalid_too_many_public_keys() {
    assert!(matches!(
        MilestonePayloadEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![[0u8; 32]; 300],
            None,
        ),
        Err(Error::MilestoneInvalidPublicKeyCount(300))
    ));
}

#[test]
fn new_valid_sorted_unique_public_keys() {
    assert!(MilestonePayloadEssence::new(
        MilestoneIndex(0),
        0,
        rand_parents(),
        [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH],
        0,
        0,
        vec![[0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32]],
        None,
    )
    .is_ok());
}

#[test]
fn new_invalid_sorted_not_unique_public_keys() {
    assert!(matches!(
        MilestonePayloadEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
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
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![[0; 32], [1; 32], [3; 32], [2; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32]],
            None,
        ),
        Err(Error::MilestonePublicKeysNotUniqueSorted)
    ));
}

#[test]
fn new_invalid_payload_kind() {
    assert!(matches!(
        MilestonePayloadEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![[0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32]],
            Some(Payload::Indexation(Box::new(
                IndexationPayload::new(&rand_bytes_array::<32>(), &[]).unwrap()
            ))),
        ),
        Err(Error::InvalidPayloadKind(2))
    ));
}

#[test]
fn getters() {
    let index = rand::milestone::rand_milestone_index();
    let timestamp = rand::number::rand_number::<u64>();
    let parents = rand_parents();
    let merkel_proof = [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH];
    let next_pow_score = 0;
    let next_pow_score_milestone_index = 0;
    let public_keys = vec![
        [0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32],
    ];

    let receipt = Some(Payload::Receipt(Box::new(
        ReceiptPayload::new(
            index,
            true,
            vec![rand::receipt::rand_migrated_funds_entry()],
            Payload::TreasuryTransaction(Box::new(
                TreasuryTransactionPayload::new(
                    Input::Treasury(TreasuryInput::new(rand::milestone::rand_milestone_id())),
                    Output::Treasury(TreasuryOutput::new(1_000_000).unwrap()),
                )
                .unwrap(),
            )),
        )
        .unwrap(),
    )));

    let milestone_payload = MilestonePayloadEssence::new(
        index,
        timestamp,
        parents.clone(),
        merkel_proof,
        next_pow_score,
        next_pow_score_milestone_index,
        public_keys.clone(),
        receipt.clone(),
    )
    .unwrap();

    assert_eq!(milestone_payload.index(), index);
    assert_eq!(milestone_payload.timestamp(), timestamp);
    assert_eq!(*milestone_payload.parents(), parents);
    assert_eq!(milestone_payload.merkle_proof(), merkel_proof);
    assert_eq!(milestone_payload.next_pow_score(), next_pow_score);
    assert_eq!(
        milestone_payload.next_pow_score_milestone_index(),
        next_pow_score_milestone_index
    );
    assert_eq!(*milestone_payload.public_keys(), public_keys);
    assert_eq!(*milestone_payload.receipt().unwrap(), receipt.unwrap());
}

#[test]
fn pack_unpack_valid() {
    let milestone_payload = MilestonePayloadEssence::new(
        MilestoneIndex(0),
        0,
        rand_parents(),
        [0; MilestonePayloadEssence::MERKLE_PROOF_LENGTH],
        0,
        0,
        vec![
            [0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32],
        ],
        None,
    )
    .unwrap();

    let packed = milestone_payload.pack_new();

    assert_eq!(
        MilestonePayloadEssence::unpack(&mut packed.as_slice()).unwrap(),
        milestone_payload,
    );
}

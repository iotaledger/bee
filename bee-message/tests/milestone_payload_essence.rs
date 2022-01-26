// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    input::{Input, TreasuryInput},
    milestone::MilestoneIndex,
    output::{Output, TreasuryOutput},
    payload::{milestone::MilestoneEssence, Payload, ReceiptPayload, TaggedDataPayload, TreasuryTransactionPayload},
    Error,
};
use bee_test::rand::{self, bytes::rand_bytes, parents::rand_parents};

use packable::{bounded::TryIntoBoundedU8Error, PackableExt};

#[test]
fn new_invalid_pow_score_non_zero() {
    assert!(matches!(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            4242,
            vec![[0u8; 32]; 1],
            None,
        ),
        Err(Error::InvalidPowScoreValues { nps: 0, npsmi: 4242 })
    ));
}

#[test]
fn new_invalid_pow_score_lower_than_index() {
    assert!(matches!(
        MilestoneEssence::new(
            MilestoneIndex(4242),
            0,
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            4000,
            4241,
            vec![[0u8; 32]; 1],
            None,
        ),
        Err(Error::InvalidPowScoreValues { nps: 4000, npsmi: 4241 })
    ));
}

#[test]
fn new_invalid_no_public_key() {
    assert!(matches!(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![],
            None,
        ),
        Err(Error::MilestoneInvalidPublicKeyCount(TryIntoBoundedU8Error::Invalid(0)))
    ));
}

#[test]
fn new_invalid_too_many_public_keys() {
    assert!(matches!(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![[0u8; 32]; 300],
            None,
        ),
        Err(Error::MilestoneInvalidPublicKeyCount(TryIntoBoundedU8Error::Truncated(
            300
        )))
    ));
}

#[test]
fn new_valid_sorted_unique_public_keys() {
    assert!(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![
                [0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32]
            ],
            None,
        )
        .is_ok()
    );
}

#[test]
fn new_invalid_sorted_not_unique_public_keys() {
    assert!(matches!(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![
                [0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [4; 32], [6; 32], [7; 32], [8; 32], [9; 32]
            ],
            None,
        ),
        Err(Error::MilestonePublicKeysNotUniqueSorted)
    ));
}

#[test]
fn new_invalid_not_sorted_unique_public_keys() {
    assert!(matches!(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![
                [0; 32], [1; 32], [3; 32], [2; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32]
            ],
            None,
        ),
        Err(Error::MilestonePublicKeysNotUniqueSorted)
    ));
}

#[test]
fn new_invalid_payload_kind() {
    assert!(matches!(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![
                [0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32]
            ],
            Some(Payload::TaggedData(Box::new(
                TaggedDataPayload::new(rand_bytes(32), vec![]).unwrap()
            ))),
        ),
        Err(Error::InvalidPayloadKind(5))
    ));
}

#[test]
fn getters() {
    let index = rand::milestone::rand_milestone_index();
    let timestamp = rand::number::rand_number::<u64>();
    let parents = rand_parents();
    let merkel_proof = [0; MilestoneEssence::MERKLE_PROOF_LENGTH];
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

    let milestone_payload = MilestoneEssence::new(
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
    let milestone_payload = MilestoneEssence::new(
        MilestoneIndex(0),
        0,
        rand_parents(),
        [0; MilestoneEssence::MERKLE_PROOF_LENGTH],
        0,
        0,
        vec![
            [0; 32], [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], [7; 32], [8; 32], [9; 32],
        ],
        None,
    )
    .unwrap();

    let packed = milestone_payload.pack_to_vec();

    assert_eq!(
        MilestoneEssence::unpack_verified(&mut packed.as_slice()).unwrap(),
        milestone_payload,
    );
}

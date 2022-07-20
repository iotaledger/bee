// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    input::TreasuryInput,
    output::TreasuryOutput,
    payload::{
        milestone::{
            MilestoneEssence, MilestoneIndex, MilestoneOption, MilestoneOptions, ParametersMilestoneOption,
            ReceiptMilestoneOption,
        },
        TreasuryTransactionPayload,
    },
    rand::{
        self,
        bytes::rand_bytes,
        milestone::{rand_merkle_root, rand_milestone_id, rand_milestone_index},
        number::{rand_number, rand_number_range},
        parents::rand_parents,
    },
};
use packable::PackableExt;

#[test]
fn new_valid() {
    assert!(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_milestone_id(),
            rand_parents(),
            rand_merkle_root(),
            rand_merkle_root(),
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
    let inclusion_merkle_root = rand_merkle_root();
    let applied_merkle_root = rand_merkle_root();
    let target_milestone_index = rand_milestone_index();
    let protocol_version = rand_number();
    let binary_parameters =
        rand_bytes(rand_number_range(ParametersMilestoneOption::BINARY_PARAMETERS_LENGTH_RANGE) as usize);
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
        MilestoneOption::Parameters(
            ParametersMilestoneOption::new(target_milestone_index, protocol_version, binary_parameters.clone())
                .unwrap(),
        ),
    ])
    .unwrap();

    let milestone_payload = MilestoneEssence::new(
        index,
        timestamp,
        previous_milestone_id,
        parents.clone(),
        inclusion_merkle_root,
        applied_merkle_root,
        vec![],
        options,
    )
    .unwrap();

    assert_eq!(milestone_payload.index(), index);
    assert_eq!(milestone_payload.timestamp(), timestamp);
    assert_eq!(milestone_payload.previous_milestone_id(), &previous_milestone_id);
    assert_eq!(*milestone_payload.parents(), parents);
    assert_eq!(*milestone_payload.inclusion_merkle_root(), inclusion_merkle_root);
    assert_eq!(*milestone_payload.applied_merkle_root(), applied_merkle_root);
    assert_eq!(
        milestone_payload
            .options()
            .parameters()
            .unwrap()
            .target_milestone_index(),
        target_milestone_index
    );
    assert_eq!(
        milestone_payload.options().parameters().unwrap().protocol_version(),
        protocol_version
    );
    assert_eq!(
        milestone_payload.options().parameters().unwrap().binary_parameters(),
        binary_parameters
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
        rand_merkle_root(),
        rand_merkle_root(),
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

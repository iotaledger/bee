// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_tangle::milestone_metadata::MilestoneMetadata;
use bee_test::rand::{
    block::rand_block_id,
    milestone::{rand_milestone_id, rand_milestone_metadata},
    number::rand_number,
};
use packable::PackableExt;

#[test]
fn packed_len() {
    let milestone = rand_milestone_metadata();
    assert_eq!(milestone.packed_len(), milestone.pack_to_vec().len());
    assert_eq!(milestone.packed_len(), 32 + 32 + 4);
}

#[test]
fn pack_unpack() {
    let milestone = rand_milestone_metadata();
    let packed = milestone.pack_to_vec();

    assert_eq!(
        MilestoneMetadata::unpack_verified(&mut packed.as_slice()).unwrap(),
        milestone
    );
}

#[test]
fn getters() {
    let block_id = rand_block_id();
    let milestone_id = rand_milestone_id();
    let timestamp = rand_number::<u32>();
    let milestone = MilestoneMetadata::new(block_id, milestone_id, timestamp);

    assert_eq!(block_id, *milestone.block_id());
    assert_eq!(milestone_id, *milestone.milestone_id());
    assert_eq!(timestamp, milestone.timestamp());
}

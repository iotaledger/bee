// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::milestone::{MerkleRoot, MilestoneId, MilestoneIndex},
    rand::{bytes::rand_bytes_array, number::rand_number},
};

/// Generates a random milestone index.
pub fn rand_milestone_index() -> MilestoneIndex {
    MilestoneIndex::from(rand_number::<u32>())
}

/// Generates a random milestone id.
pub fn rand_milestone_id() -> MilestoneId {
    MilestoneId::new(rand_bytes_array())
}

/// Generates a random merkle root.
pub fn rand_merkle_root() -> MerkleRoot {
    MerkleRoot::from(rand_bytes_array())
}

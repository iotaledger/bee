// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::payload::milestone::{MerkleRoot, MilestoneId, MilestoneIndex};
use bee_tangle::milestone_metadata::MilestoneMetadata;

use crate::rand::{block::rand_block_id, bytes::rand_bytes_array, number::rand_number};

/// Generates a random milestone index.
pub fn rand_milestone_index() -> MilestoneIndex {
    MilestoneIndex::from(rand_number::<u32>())
}

/// Generates a random milestone id.
pub fn rand_milestone_id() -> MilestoneId {
    MilestoneId::new(rand_bytes_array())
}

/// Generates a random milestone metadata.
pub fn rand_milestone_metadata() -> MilestoneMetadata {
    MilestoneMetadata::new(rand_block_id(), rand_milestone_id(), rand_number())
}

/// Generates a random merkle root.
pub fn rand_merkle_root() -> MerkleRoot {
    MerkleRoot::from(rand_bytes_array())
}

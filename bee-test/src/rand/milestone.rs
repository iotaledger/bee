// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::milestone::{MilestoneId, MilestoneIndex};
use bee_tangle::milestone_metadata::MilestoneMetadata;

use crate::rand::{bytes::rand_bytes_array, message::rand_message_id, number::rand_number};

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
    MilestoneMetadata::new(rand_message_id(), rand_milestone_id(), rand_number())
}

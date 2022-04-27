// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    milestone::Milestone,
    payload::milestone::{MilestoneId, MilestoneIndex},
};

use crate::rand::{bytes::rand_bytes_array, message::rand_message_id, number::rand_number};

/// Generates a random milestone index.
pub fn rand_milestone_index() -> MilestoneIndex {
    MilestoneIndex::from(rand_number::<u32>())
}

/// Generates a random milestone id.
pub fn rand_milestone_id() -> MilestoneId {
    MilestoneId::new(rand_bytes_array())
}

/// Generates a random milestone.
pub fn rand_milestone() -> Milestone {
    Milestone::new(rand_message_id(), rand_number())
}

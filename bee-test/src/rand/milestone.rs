// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes_32, integer::rand_integer, message::rand_message_id};

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    payload::milestone::MilestoneId,
};

/// Generates a random milestone index.
pub fn rand_milestone_index() -> MilestoneIndex {
    MilestoneIndex::from(rand_integer::<u32>())
}

/// Generates a random milestone id.
pub fn rand_milestone_id() -> MilestoneId {
    MilestoneId::new(rand_bytes_32())
}

/// Generates a random milestone.
pub fn rand_milestone() -> Milestone {
    Milestone::new(rand_message_id(), rand_integer::<u64>())
}

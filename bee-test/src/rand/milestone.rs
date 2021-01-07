// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::rand_integer, message::rand_message_id};

use bee_tangle::milestone::{Milestone, MilestoneIndex};

pub fn rand_milestone_index() -> MilestoneIndex {
    MilestoneIndex::from(rand_integer::<u32>())
}

pub fn rand_milestone() -> Milestone {
    Milestone::new(rand_message_id(), rand_integer::<u64>())
}

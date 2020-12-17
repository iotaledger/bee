// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::random_integer, message::random_message_id};

use bee_protocol::{Milestone, MilestoneIndex};

pub fn random_milestone_index() -> MilestoneIndex {
    MilestoneIndex::from(random_integer::<u32>())
}

pub fn random_milestone() -> Milestone {
    Milestone::new(random_milestone_index(), random_message_id(), random_integer::<u64>())
}

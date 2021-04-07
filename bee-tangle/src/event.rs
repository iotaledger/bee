// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::{Milestone, MilestoneIndex};

#[derive(Clone, Debug)]
pub struct LatestMilestoneChanged {
    pub index: MilestoneIndex,
    pub milestone: Milestone,
}

#[derive(Clone, Debug)]
pub struct LatestSolidMilestoneChanged {
    pub index: MilestoneIndex,
    pub milestone: Milestone,
}

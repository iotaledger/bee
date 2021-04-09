// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::{Milestone, MilestoneIndex};

#[derive(Clone)]
pub struct LatestMilestoneChanged {
    pub index: MilestoneIndex,
    pub milestone: Milestone,
}

#[derive(Clone)]
pub struct SolidMilestoneChanged {
    pub index: MilestoneIndex,
    pub milestone: Milestone,
}

#[derive(Clone)]
pub struct ConfirmedMilestoneChanged {
    pub index: MilestoneIndex,
    pub milestone: Milestone,
}

#[derive(Clone)]
pub struct SnapshotMilestoneIndexChanged(pub MilestoneIndex);

#[derive(Clone)]
pub struct PruningMilestoneIndexChanged(pub MilestoneIndex);

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::{Milestone, MilestoneIndex};

/// An event that indicates that the latest milestone has changed.
#[derive(Clone)]
pub struct LatestMilestoneChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
    /// The milestone data
    pub milestone: Milestone,
}

/// An event that indicates that a solid milestone has changed.
#[derive(Clone)]
pub struct SolidMilestoneChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
    /// The milestone data
    pub milestone: Milestone,
}

/// An event that indicates that a confirmed milestone has changed.
#[derive(Clone)]
pub struct ConfirmedMilestoneChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
    /// The milestone data
    pub milestone: Milestone,
}

/// An event that indicates that a snapshot milestone has had an index change.
#[derive(Clone)]
pub struct SnapshotMilestoneIndexChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
}

/// An event that indicates that a pruning milestone has had an index change.
#[derive(Clone)]
pub struct PruningMilestoneIndexChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
}

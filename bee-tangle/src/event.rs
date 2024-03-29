// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::payload::milestone::MilestoneIndex;

use crate::milestone_metadata::MilestoneMetadata;

/// An event that indicates that the latest milestone has changed.
#[derive(Clone)]
pub struct LatestMilestoneChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
    /// The milestone metadata
    pub milestone: MilestoneMetadata,
}

/// An event that indicates that the solid milestone has changed.
#[derive(Clone)]
pub struct SolidMilestoneChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
    /// The milestone metadata
    pub milestone: MilestoneMetadata,
}

/// An event that indicates that the confirmed milestone has changed.
#[derive(Clone)]
pub struct ConfirmedMilestoneChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
    /// The milestone metadata
    pub milestone: MilestoneMetadata,
}

/// An event that indicates that the snapshot milestone has changed.
#[derive(Clone)]
pub struct SnapshotMilestoneIndexChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
}

/// An event that indicates that the pruning milestone has changed.
#[derive(Clone)]
pub struct PruningMilestoneIndexChanged {
    /// The index of the milestone
    pub index: MilestoneIndex,
}

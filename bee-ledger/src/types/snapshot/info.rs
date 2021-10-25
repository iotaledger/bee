// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::MilestoneIndex;
use bee_packable::Packable;

/// Snapshot information to be stored.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
pub struct SnapshotInfo {
    network_id: u64,
    snapshot_index: MilestoneIndex,
    entry_point_index: MilestoneIndex,
    pruning_index: MilestoneIndex,
    timestamp: u64,
}

impl SnapshotInfo {
    /// Creates a new `SnapshotInfo`.
    pub fn new(
        network_id: u64,
        snapshot_index: MilestoneIndex,
        entry_point_index: MilestoneIndex,
        pruning_index: MilestoneIndex,
        timestamp: u64,
    ) -> Self {
        Self {
            network_id,
            snapshot_index,
            entry_point_index,
            pruning_index,
            timestamp,
        }
    }

    /// Returns the network identifier of a `SnapshotInfo`.
    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    /// Returns the snapshot index of a `SnapshotInfo`.
    pub fn snapshot_index(&self) -> MilestoneIndex {
        self.snapshot_index
    }

    /// Updates the snapshot index of a `SnapshotInfo`.
    pub fn update_snapshot_index(&mut self, index: MilestoneIndex) {
        self.snapshot_index = index;
    }

    /// Returns the entry point index of a `SnapshotInfo`.
    pub fn entry_point_index(&self) -> MilestoneIndex {
        self.entry_point_index
    }

    /// Updates the entry point index of a `SnapshotInfo`.
    pub fn update_entry_point_index(&mut self, index: MilestoneIndex) {
        self.entry_point_index = index;
    }

    /// Returns the pruning index of a `SnapshotInfo`.
    pub fn pruning_index(&self) -> MilestoneIndex {
        self.pruning_index
    }

    /// Updates the pruning index of a `SnapshotInfo`.
    pub fn update_pruning_index(&mut self, index: MilestoneIndex) {
        self.pruning_index = index;
    }

    /// Returns the timestamp of a `SnapshotInfo`.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Updates the timestamp of a `SnapshotInfo`.
    pub fn update_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }
}

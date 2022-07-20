// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::rand::{milestone::rand_milestone_index, number::rand_number};

use crate::snapshot::SnapshotInfo;

/// Generates a random snapshot info.
pub fn rand_snapshot_info() -> SnapshotInfo {
    SnapshotInfo::new(
        rand_number(),
        rand_milestone_index(),
        rand_milestone_index(),
        rand_milestone_index(),
        rand_number(),
    )
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::rand_integer, milestone::rand_milestone_index};

use bee_ledger::types::snapshot::SnapshotInfo;

pub fn rand_snapshot_info() -> SnapshotInfo {
    SnapshotInfo::new(
        rand_integer(),
        rand_milestone_index(),
        rand_milestone_index(),
        rand_milestone_index(),
        rand_integer(),
    )
}

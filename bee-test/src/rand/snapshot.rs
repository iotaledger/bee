// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::integer::rand_integer;

use bee_snapshot::info::SnapshotInfo;

pub fn rand_snapshot_info() -> SnapshotInfo {
    SnapshotInfo::new(
        rand_integer(),
        rand_integer(),
        rand_integer(),
        rand_integer(),
        rand_integer(),
    )
}

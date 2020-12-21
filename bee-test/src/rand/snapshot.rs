// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::integer::random_integer;

use bee_snapshot::info::SnapshotInfo;

pub fn random_snapshot_info() -> SnapshotInfo {
    SnapshotInfo::new(
        random_integer(),
        random_integer(),
        random_integer(),
        random_integer(),
        random_integer(),
    )
}

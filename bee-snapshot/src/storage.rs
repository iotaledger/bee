// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::info::SnapshotInfo;

use bee_storage::{access::Fetch, storage};

pub trait Backend: storage::Backend + Fetch<(), SnapshotInfo> {}

impl<T> Backend for T where T: storage::Backend + Fetch<(), SnapshotInfo> {}

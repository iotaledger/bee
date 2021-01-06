// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::info::SnapshotInfo;

use bee_storage::{access::Fetch, backend};

pub trait StorageBackend: backend::StorageBackend + Fetch<(), SnapshotInfo> {}

impl<T> StorageBackend for T where T: backend::StorageBackend + Fetch<(), SnapshotInfo> {}

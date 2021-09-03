// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::packable::Packable;

/// Version of the storage.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Packable)]
pub struct StorageVersion(pub u64);

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::Packable;

/// Version of the storage.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Packable)]
pub struct StorageVersion(pub u64);

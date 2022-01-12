// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Version of the storage.
#[derive(Debug, Copy, Clone, Eq, PartialEq, bee_packable::Packable)]
pub struct StorageVersion(pub u64);

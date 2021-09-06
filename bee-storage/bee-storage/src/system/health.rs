// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Defines a type to represent different health states in which the storage backend can be.

use bee_packable::Packable;

/// Represents different health states for a `StorageBackend`.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Packable)]
#[packable(tag_type = u8)]
pub enum StorageHealth {
    /// The storage is in a healthy state.
    Healthy = 0,
    /// The storage is running and the health status is idle.
    Idle = 1,
    /// The storage has been corrupted.
    Corrupted = 2,
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types related to the state of the storage itself.

mod health;
mod version;

pub use health::StorageHealth;
pub use version::StorageVersion;

use bee_packable::Packable;

/// Key used to store the system version.
pub const SYSTEM_VERSION_KEY: u8 = 0;
/// Key used to store the system health.
pub const SYSTEM_HEALTH_KEY: u8 = 1;

/// System-related information.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Packable)]
#[packable(tag_type = u8)]
pub enum System {
    /// The current version of the storage.
    #[packable(tag = SYSTEM_VERSION_KEY)]
    Version(StorageVersion),
    /// The health status of the storage.
    #[packable(tag = SYSTEM_HEALTH_KEY)]
    Health(StorageHealth),
}

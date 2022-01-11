// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types related to the state of the storage itself.

mod health;
mod version;

pub use health::{Error as StorageHealthError, StorageHealth};
pub use version::StorageVersion;

use core::convert::Infallible;

/// Key used to store the system version.
pub const SYSTEM_VERSION_KEY: u8 = 0;
/// Key used to store the system health.
pub const SYSTEM_HEALTH_KEY: u8 = 1;

/// Errors to be raised if packing/unpacking `System` fails.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error.
    #[error("I/O error happened: {0}")]
    Io(#[from] std::io::Error),
    /// Packing/unpacking the `System::Health` variant failed.
    #[error("Storage health error: {0}")]
    Health(#[from] StorageHealthError),
    /// Found an invalid key while unpacking a `System` value.
    #[error("Unknown system key: {0}")]
    UnknownSystemKey(u8),
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

/// System-related information.
#[derive(Debug, Copy, Clone, Eq, PartialEq, bee_packable::Packable)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::UnknownSystemKey)]
pub enum System {
    /// The current version of the storage.
    #[packable(tag = SYSTEM_VERSION_KEY)]
    Version(StorageVersion),
    /// The health status of the storage.
    #[packable(tag = SYSTEM_HEALTH_KEY)]
    Health(StorageHealth),
}

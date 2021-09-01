// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types related to the state of the storage itself.

mod health;
mod version;

use core::convert::Infallible;

pub use health::{Error as StorageHealthError, StorageHealth};
pub use version::StorageVersion;

use bee_packable::{
    coerce::*,
    error::{PackError, UnpackError},
    packable::{Packable, Packer, Unpacker},
};

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

/// System-related information.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum System {
    /// The current version of the storage.
    Version(StorageVersion),
    /// The health status of the storage.
    Health(StorageHealth),
}

impl Packable for System {
    type PackError = Infallible;
    type UnpackError = Error;

    fn packed_len(&self) -> usize {
        match self {
            System::Version(version) => SYSTEM_VERSION_KEY.packed_len() + version.packed_len(),
            System::Health(health) => SYSTEM_HEALTH_KEY.packed_len() + health.packed_len(),
        }
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        match self {
            System::Version(version) => {
                SYSTEM_VERSION_KEY.pack(packer).infallible()?;
                version.pack(packer)?;
            }
            System::Health(health) => {
                SYSTEM_HEALTH_KEY.pack(packer).infallible()?;
                health.pack(packer)?;
            }
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        match u8::unpack(unpacker).infallible()? {
            SYSTEM_VERSION_KEY => Ok(System::Version(StorageVersion::unpack(unpacker).infallible()?)),
            SYSTEM_HEALTH_KEY => Ok(System::Health(StorageHealth::unpack(unpacker).coerce()?)),
            s => Err(bee_packable::UnpackError::Packable(Error::UnknownSystemKey(s))),
        }
    }
}

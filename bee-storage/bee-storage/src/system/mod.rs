// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types related to the state of the storage itself.

mod version;

pub use version::StorageVersion;

use crate::health::{Error as StorageHealthError, StorageHealth};

use bee_common::packable::{Packable, Read, Write};

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
    /// Found and invalid key while unpacking a `System` value.
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
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            System::Version(version) => SYSTEM_VERSION_KEY.packed_len() + version.packed_len(),
            System::Health(health) => SYSTEM_HEALTH_KEY.packed_len() + health.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            System::Version(version) => {
                SYSTEM_VERSION_KEY.pack(writer)?;
                version.pack(writer)?;
            }
            System::Health(health) => {
                SYSTEM_HEALTH_KEY.pack(writer)?;
                health.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        match u8::unpack_inner::<R, CHECK>(reader)? {
            SYSTEM_VERSION_KEY => Ok(System::Version(StorageVersion::unpack_inner::<R, CHECK>(reader)?)),
            SYSTEM_HEALTH_KEY => Ok(System::Health(StorageHealth::unpack_inner::<R, CHECK>(reader)?)),
            s => Err(Error::UnknownSystemKey(s)),
        }
    }
}

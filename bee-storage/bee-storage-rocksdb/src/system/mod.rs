// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod version;

pub(crate) use version::{StorageVersion, STORAGE_VERSION, STORAGE_VERSION_KEY};

pub(crate) const STORAGE_HEALTH_KEY: u8 = 1;

use bee_common::packable::{Packable, Read, Write};
use bee_storage::health::{Error as StorageHealthError, StorageHealth};

#[derive(Debug, thiserror::Error)]
pub enum SystemError {
    #[error("I/O error happened: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("")]
    Health(#[from] StorageHealthError),
    #[error("")]
    UnknownSystem(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum System {
    Version(StorageVersion),
    Health(StorageHealth),
}

impl Packable for System {
    type Error = SystemError;

    fn packed_len(&self) -> usize {
        match self {
            System::Version(version) => STORAGE_VERSION_KEY.packed_len() + version.packed_len(),
            System::Health(health) => STORAGE_HEALTH_KEY.packed_len() + health.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            System::Version(version) => {
                STORAGE_VERSION_KEY.pack(writer)?;
                version.pack(writer)?;
            }
            System::Health(health) => {
                STORAGE_HEALTH_KEY.pack(writer)?;
                health.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        match u8::unpack_inner::<R, CHECK>(reader)? {
            STORAGE_VERSION_KEY => Ok(System::Version(StorageVersion::unpack_inner::<R, CHECK>(reader)?)),
            STORAGE_HEALTH_KEY => Ok(System::Health(StorageHealth::unpack_inner::<R, CHECK>(reader)?)),
            s => Err(SystemError::UnknownSystem(s)),
        }
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod health;
mod version;

pub(crate) use health::{StorageHealth, STORAGE_HEALTH_KEY};
pub(crate) use version::{StorageVersion, STORAGE_VERSION, STORAGE_VERSION_KEY};

use bee_common::packable::{Packable, Read, Write};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum System {
    Version(StorageVersion),
    Health(StorageHealth),
}

impl Packable for System {
    type Error = std::io::Error;

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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack(reader)? {
            STORAGE_VERSION_KEY => System::Version(StorageVersion::unpack(reader)?),
            STORAGE_HEALTH_KEY => System::Health(StorageHealth::unpack(reader)?),
            _ => panic!("Unhandled system type"),
        })
    }
}

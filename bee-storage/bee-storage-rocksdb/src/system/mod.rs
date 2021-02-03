// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod health;
mod version;

use health::StorageHealth;
use version::StorageVersion;

use bee_common::packable::{Packable, Read, Write};

pub enum System {
    Version(StorageVersion),
    Health(StorageHealth),
}

impl Packable for System {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        match self {
            System::Version(version) => 0u8.packed_len() + version.packed_len(),
            System::Health(health) => 1u8.packed_len() + health.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            System::Version(version) => {
                0u8.pack(writer)?;
                version.pack(writer)?;
            }
            System::Health(health) => {
                1u8.pack(writer)?;
                health.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack(reader)? {
            0 => System::Version(StorageVersion::unpack(reader)?),
            1 => System::Health(StorageHealth::unpack(reader)?),
            _ => panic!("Unhandled system type"),
        })
    }
}

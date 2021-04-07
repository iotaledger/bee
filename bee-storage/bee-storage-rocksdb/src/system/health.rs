// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::{Packable, Read, Write};

// TODO handle panic

pub(crate) const STORAGE_HEALTH_KEY: u8 = 1;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StorageHealth {
    Healthy = 0,
    Unhealthy = 1,
}

impl Packable for StorageHealth {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (*self as u8).pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            0 => StorageHealth::Healthy,
            1 => StorageHealth::Unhealthy,
            _ => panic!("Unhandled"),
        })
    }
}

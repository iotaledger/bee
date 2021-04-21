// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::{Packable, Read, Write};

pub(crate) const STORAGE_VERSION_KEY: u8 = 0;
pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion(6);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StorageVersion(pub u16);

impl Packable for StorageVersion {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(u16::unpack_inner::<R, CHECK>(reader)?))
    }
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::convert::Infallible;

use bee_packable::{
    error::{PackError, UnpackError},
    packable::{Packable, Packer, Unpacker}
};

/// Version of the storage.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StorageVersion(pub u64);

impl Packable for StorageVersion {
    type PackError = Infallible;
    type UnpackError = Infallible;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.0.pack(packer)
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(Self(u64::unpack(unpacker)?))
    }
}

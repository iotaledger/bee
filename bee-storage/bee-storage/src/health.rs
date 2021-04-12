// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Defines a type to represent different health states in which the storage backend can be.

use bee_common::packable::{Packable, Read, Write};

// TODO handle panic

/// Represents different health states for a `StorageBackend`.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StorageHealth {
    /// Healthy backend state.
    Healthy = 0,
    /// Idle backend state.
    Idle = 1,
    /// Corrupted backend state.
    Corrupted = 2,
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
            1 => StorageHealth::Idle,
            2 => StorageHealth::Corrupted,
            _ => panic!("Unhandled"),
        })
    }
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Defines a type to represent different health states in which the storage backend can be.

use bee_common::packable::{Packable, Read, Write};

/// Errors related to storage health statuses.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error happened.
    #[error("I/O error happened: {0:?}")]
    Io(#[from] std::io::Error),
    /// Unknown storage health variant.
    #[error("Unknown storage health variant: {0}")]
    UnknownHealth(u8),
}

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
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (*self as u8).pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        match u8::unpack_inner::<R, CHECK>(reader)? {
            0 => Ok(StorageHealth::Healthy),
            1 => Ok(StorageHealth::Idle),
            2 => Ok(StorageHealth::Corrupted),
            h => Err(Error::UnknownHealth(h)),
        }
    }
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Defines a type to represent different health states in which the storage backend can be.

use bee_packable::{
    error::{PackError, UnpackError},
    packable::{Packable, Packer, Unpacker}
};

use bee_packable::coerce::*;

use core::convert::Infallible;

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
    /// The storage is in a healthy state.
    Healthy = 0,
    /// The storage is running and the health status is idle.
    Idle = 1,
    /// The storage has been corrupted.
    Corrupted = 2,
}

impl Packable for StorageHealth {
    type PackError = Infallible;
    type UnpackError = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        (*self as u8).pack(packer).infallible()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        match u8::unpack(unpacker).infallible()? {
            0 => Ok(StorageHealth::Healthy),
            1 => Ok(StorageHealth::Idle),
            2 => Ok(StorageHealth::Corrupted),
            h => Err(bee_packable::UnpackError::Packable(Error::UnknownHealth(h))),
        }
    }
}

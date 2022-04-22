// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Defines a type to represent different health states in which the storage backend can be.

use core::convert::Infallible;

/// Errors related to storage health statuses.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error happened.
    #[error("i/o error happened: {0:?}")]
    Io(#[from] std::io::Error),
    /// Unknown storage health variant.
    #[error("unknown storage health variant: {0}")]
    UnknownHealth(u8),
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

/// Represents different health states for a `StorageBackend`.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, packable::Packable)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::UnknownHealth)]
pub enum StorageHealth {
    /// The storage is in a healthy state.
    Healthy = 0,
    /// The storage is running and the health status is idle.
    Idle = 1,
    /// The storage has been corrupted.
    Corrupted = 2,
}

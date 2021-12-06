// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module to unpack any value that implements [`Packable`](crate::Packable).
//!
//! The [`Unpacker`] trait represents types that can be used to read bytes from it. It can be thought as a `no_std`
//! friendly alternative to the [`Read`](std::io::Read) trait.

#[cfg(feature = "io")]
mod io;
mod slice;

#[cfg(feature = "io")]
pub use io::IoUnpacker;

/// A type that can unpack any value that implements [`Packable`](crate::Packable).
pub trait Unpacker: Sized {
    /// An error type representing any error related to reading bytes.
    type Error;

    /// Reads a sequence of bytes from the [`Unpacker`]. This sequence must be long enough to fill `bytes` completely.
    /// This method **must** fail if the unpacker does not have enough bytes to fulfill the request.
    fn unpack_bytes<B: AsMut<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error>;

    #[inline]
    /// Returns the **exact** number of bytes that the [`Unpacker`] has left.
    fn remaining_bytes(&self) -> Option<usize> {
        None
    }
}

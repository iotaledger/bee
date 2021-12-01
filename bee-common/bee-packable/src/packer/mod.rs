// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module to pack any value that implements [`Packable`](crate::Packable).
//!
//! The [`Packer`] trait represents types that can be used to write bytes into it. It can be thought as a `no_std`
//! friendly alternative to the [`Write`](std::io::Write) trait.

#[cfg(feature = "io")]
mod io;
mod len;
mod slice;
mod vec;

pub(crate) use len::LenPacker;

#[cfg(feature = "io")]
pub use io::IoPacker;
pub use slice::SlicePacker;

/// A type that can pack any value that implements [`Packable`](crate::Packable).
pub trait Packer {
    /// An error type representing any error related to writing bytes.
    type Error;

    /// Writes a sequence of bytes into the [`Packer`]. The totality of `bytes` must be written into the packer.
    /// This method **must** fail if the packer does not have enough space to fulfill the request.
    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error>;
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module to pack any value that implements [`Packable`](crate::Packable).

#[cfg(feature = "io")]
mod io;
mod len;
mod slice;
mod vec;

pub(crate) use len::LenPacker;

pub use slice::SlicePacker;
pub use vec::VecPacker;

/// A type that can pack any value that implements [`Packable`](crate::Packable).
pub trait Packer {
    /// The error type that can be returned if some error occurs while packing.
    type Error;

    /// Packs a sequence of bytes into the [`Packer`].
    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error>;
}

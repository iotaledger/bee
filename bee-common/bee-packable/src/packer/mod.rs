// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module to pack any value that implements `Packable`.

#[cfg(feature = "io")]
mod io;
mod vec;

pub use vec::VecPacker;

/// A type that can pack any value that implements `Packable`.
pub trait Packer {
    /// The error type that can be returned if some error occurs while packing.
    type Error;

    /// Pack a sequence of bytes into the `Packer`.
    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error>;
}

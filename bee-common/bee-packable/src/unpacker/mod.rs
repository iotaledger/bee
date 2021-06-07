// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module to unpack any value that implements `Packable`.

#[cfg(feature = "io")]
mod io;
mod slice;

pub use slice::{SliceUnpacker, UnexpectedEOF};

/// A type that can unpack any value that implements `Packable`.
pub trait Unpacker: Sized {
    /// The error type that can be returned if some error occurs while unpacking.
    type Error;

    /// Unpack a sequence of bytes from the `Unpacker`.
    fn unpack_bytes<B: AsMut<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error>;
}

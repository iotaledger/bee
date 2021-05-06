// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::Packable;

/// A type that can unpack any value that implements `Packer`.
pub trait Unpacker {
    /// The error type that can be returned if some error occurs while unpacking.
    type Error: UnpackError;

    /// Unpack a statically-sized sequence of bytes from the `Unpacker`.
    fn unpack_exact_bytes<const N: usize>(&mut self) -> Result<&[u8; N], Self::Error>;

    /// Unpack a sequence of bytes from the `Unpacker`.
    fn unpack_bytes(&mut self, n: usize) -> Result<&[u8], Self::Error>;
}

/// A type that represents errors with the unpacking format as well as with the unpacking process itself.
pub trait UnpackError {
    /// Raised when there is an invalid variant identifier for the enum `P` while unpacking.
    fn invalid_variant<P: Packable>(identifier: u64) -> Self;
}

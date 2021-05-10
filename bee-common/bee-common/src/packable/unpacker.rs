// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::UnpackError;

/// A type that can unpack any value that implements `Packer`.
pub trait Unpacker {
    /// The error type that can be returned if some error occurs while unpacking.
    type Error: UnpackError;

    /// Unpack a sequence of bytes from the `Unpacker`.
    fn unpack_bytes(&mut self, slice: &mut [u8]) -> Result<(), Self::Error>;
}

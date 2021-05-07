// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// A type that can pack any value that implements `Packer`.
pub trait Packer {
    /// The error type that can be returned if some error occurs while packing.
    type Error;

    /// Pack a sequence of bytes into the `Packer`.
    fn pack_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

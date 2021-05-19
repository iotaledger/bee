// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use super::unpacker::SliceUnpacker;

use alloc::vec::Vec;
use core::convert::Infallible;

/// A type that can pack any value that implements `Packable`.
pub trait Packer {
    /// The error type that can be returned if some error occurs while packing.
    type Error;

    /// Pack a sequence of bytes into the `Packer`.
    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error>;
}

/// A `Packer` backed by a `Vec<u8>`.
#[derive(Default)]
pub struct VecPacker(Vec<u8>);

impl VecPacker {
    /// Create a new, empty packer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use the backing `Vec<u8>` to create an `Unpacker`.
    pub fn as_slice(&self) -> SliceUnpacker<'_> {
        SliceUnpacker::new(self.0.as_slice())
    }

    /// Return the number of packed bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return `true` if no bytes have been packed yet.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Packer for VecPacker {
    type Error = Infallible;

    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error> {
        self.0.extend_from_slice(bytes.as_ref());
        Ok(())
    }
}

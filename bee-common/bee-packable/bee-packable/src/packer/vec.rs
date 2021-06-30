// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::{packer::Packer, unpacker::SliceUnpacker};

use alloc::vec::Vec;
use core::convert::Infallible;

/// A `Packer` backed by a `Vec<u8>`.
#[derive(Default)]
pub struct VecPacker(Vec<u8>);

impl VecPacker {
    /// Creates a new, empty packer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty packer with an initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Uses the backing `Vec<u8>` to create an `Unpacker`.
    pub fn as_slice(&self) -> SliceUnpacker<'_> {
        SliceUnpacker::new(self.0.as_slice())
    }

    /// Consumes the `VecPacker` and returns the inner `Vec<u8>`.
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }

    /// Returns the number of packed bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if no bytes have been packed yet.
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

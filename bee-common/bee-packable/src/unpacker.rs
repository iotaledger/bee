// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module to unpack any value that implements `Packable`.

#[cfg(feature = "io")]
extern crate std;

#[cfg(feature = "io")]
use std::io::{self, Read};

/// A type that can unpack any value that implements `Packable`.
pub trait Unpacker: Sized {
    /// The error type that can be returned if some error occurs while unpacking.
    type Error;

    /// Unpack a sequence of bytes from the `Unpacker`.
    fn unpack_bytes<B: AsMut<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error>;
}

/// A `Unpacker` backed by a `&[u8]`.
pub struct SliceUnpacker<'u>(&'u [u8]);

impl<'u> SliceUnpacker<'u> {
    /// Create a new unpacker from a byte slice.
    pub fn new(slice: &'u [u8]) -> Self {
        Self(slice)
    }
}

/// Error type to be raised when `SliceUnpacker` does not have enough bytes to unpack something.
#[derive(Debug)]
pub struct UnexpectedEOF {
    /// The required number of bytes.
    pub required: usize,
    /// THe number of bytes the unpacker had.
    pub had: usize,
}

impl<'u> Unpacker for SliceUnpacker<'u> {
    type Error = UnexpectedEOF;

    fn unpack_bytes<B: AsMut<[u8]>>(&mut self, mut bytes: B) -> Result<(), Self::Error> {
        let slice = bytes.as_mut();
        let len = slice.len();

        if self.0.len() >= len {
            let (head, tail) = self.0.split_at(len);
            self.0 = tail;
            slice.copy_from_slice(head);
            Ok(())
        } else {
            Err(UnexpectedEOF {
                required: len,
                had: self.0.len(),
            })
        }
    }
}

#[cfg(feature = "io")]
impl<R: Read> Unpacker for R {
    type Error = io::Error;

    fn unpack_bytes<B: AsMut<[u8]>>(&mut self, mut bytes: B) -> Result<(), Self::Error> {
        self.read_exact(bytes.as_mut())
    }
}

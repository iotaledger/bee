// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::UnexpectedEOF, packer::Packer};

/// A [`Packer`] backed by a `&mut [u8]`.
pub struct SlicePacker<'a> {
    slice: &'a mut [u8],
    offset: usize,
}

impl<'a> SlicePacker<'a> {
    /// Creates a new [`SlicePacker`] from a `&mut [u8]`.
    pub fn new(slice: &'a mut [u8]) -> Self {
        Self { slice, offset: 0 }
    }
}

impl<'a> Packer for SlicePacker<'a> {
    type Error = UnexpectedEOF;

    fn pack_bytes<B: AsRef<[u8]>>(&mut self, bytes: B) -> Result<(), Self::Error> {
        let bytes = bytes.as_ref();
        let len = bytes.len();

        match self.slice.get_mut(self.offset..self.offset + len) {
            Some(slice) => {
                slice.copy_from_slice(bytes);
                self.offset += len;

                Ok(())
            }
            None => Err(UnexpectedEOF {
                required: len,
                had: self.slice.len() - self.offset,
            }),
        }
    }
}

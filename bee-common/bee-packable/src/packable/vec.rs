// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

pub use crate::{
    error::{PackError, UnknownTagError, UnpackError},
    packer::{Packer, VecPacker},
    unpacker::{SliceUnpacker, UnexpectedEOF, Unpacker},
    Packable,
};

use alloc::vec::Vec;

impl<T: Packable> Packable for Vec<T> {
    type PackError = T::PackError;
    type UnpackError = T::UnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        // The length of any dynamically-sized sequence must be prefixed.
        self.len().pack(packer).map_err(PackError::infallible)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn packed_len(&self) -> usize {
        0usize.packed_len() + self.iter().map(T::packed_len).sum::<usize>()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        // The length of any dynamically-sized sequence must be prefixed.
        let len = usize::unpack(unpacker).map_err(UnpackError::infallible)?;

        let mut vec = Self::with_capacity(len);

        for _ in 0..len {
            let item = T::unpack(unpacker)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

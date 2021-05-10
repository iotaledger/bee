// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use super::{Packable, Packer, UnpackError, Unpacker};

impl<T: Packable> Packable for Vec<T> {
    type Error = T::Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        // The length of any dynamically-sized sequence must be prefixed.
        self.len().pack(packer)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        // The length of any dynamically-sized sequence must be prefixed.
        let len = unpacker.unpack_infallible::<usize>().map_err(UnpackError::Unpacker)?;

        let mut vec = Self::with_capacity(len);

        for _ in 0..len {
            let item = T::unpack(unpacker)?;
            vec.push(item);
        }

        Ok(vec)
    }

    fn packed_len(&self) -> usize {
        0usize.packed_len() + self.iter().map(T::packed_len).sum::<usize>()
    }
}

impl<T: Packable> Packable for Box<[T]> {
    type Error = T::Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        // The length of any dynamically-sized sequence must be prefixed.
        self.len().pack(packer)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
        Ok(Vec::<T>::unpack(unpacker)?.into_boxed_slice())
    }

    fn packed_len(&self) -> usize {
        0usize.packed_len() + self.iter().map(T::packed_len).sum::<usize>()
    }
}

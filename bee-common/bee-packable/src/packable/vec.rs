// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::{
    error::{PackError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use alloc::vec::Vec;

impl<T: Packable> Packable for Vec<T> {
    type PackError = T::PackError;
    type UnpackError = T::UnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        // The length of any dynamically-sized sequence must be prefixed.
        (self.len() as u64).pack(packer).map_err(PackError::infallible)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn packed_len(&self) -> usize {
        0u64.packed_len() + self.iter().map(T::packed_len).sum::<usize>()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        // The length of any dynamically-sized sequence must be prefixed.
        let len = u64::unpack(unpacker).map_err(UnpackError::infallible)?;

        let mut vec = Self::with_capacity(len as usize);

        for _ in 0..len {
            let item = T::unpack(unpacker)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

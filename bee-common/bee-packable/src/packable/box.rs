// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::{
    coerce::CoerceInfallible,
    error::{PackError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use alloc::{boxed::Box, vec::Vec};
use core::ops::Deref;

impl<T: Packable> Packable for Box<T> {
    type PackError = T::PackError;
    type UnpackError = T::UnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.deref().pack(packer)?;

        Ok(())
    }

    fn packed_len(&self) -> usize {
        self.deref().packed_len()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(Box::new(T::unpack(unpacker)?))
    }
}

impl<T: Packable> Packable for Box<[T]> {
    type PackError = T::PackError;
    type UnpackError = T::UnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        // The length of any dynamically-sized sequence must be prefixed.
        (self.len() as u64).pack(packer).infallible()?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn packed_len(&self) -> usize {
        0u64.packed_len() + self.iter().map(T::packed_len).sum::<usize>()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(Vec::<T>::unpack(unpacker)?.into_boxed_slice())
    }
}

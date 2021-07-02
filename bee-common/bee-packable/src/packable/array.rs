// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{PackError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

impl<T: Packable, const N: usize> Packable for [T; N] {
    type PackError = T::PackError;
    type UnpackError = T::UnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn packed_len(&self) -> usize {
        self.iter().map(T::packed_len).sum::<usize>()
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        use core::mem::MaybeUninit;

        // Safety: an uninitialized array of `MaybeUninit`s is safe to be considered initialized.
        // FIXME: replace with [`uninit_array`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.uninit_array)
        // when stabilized.
        let mut array = unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };

        for item in array.iter_mut() {
            let unpacked = T::unpack(unpacker)?;
            // Safety: each `item` is only visited once so we are never overwriting nor dropping
            // values that are already initialized.
            unsafe {
                item.as_mut_ptr().write(unpacked);
            }
        }

        // Safety: We traversed the whole array and initialized every item.
        // FIXME: replace with [`array_assume_init`](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.array_assume_init)
        // when stabilized.
        Ok(unsafe { (&array as *const [MaybeUninit<T>; N] as *const Self).read() })
    }
}

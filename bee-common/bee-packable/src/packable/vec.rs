// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use alloc::vec::Vec;

impl<T: Packable> Packable for Vec<T> {
    type UnpackError = T::UnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        // The length of any dynamically-sized sequence must be prefixed.
        (self.len() as u64).pack(packer)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        // The length of any dynamically-sized sequence must be prefixed.
        let len = u64::unpack::<_, VERIFY>(unpacker).infallible()?;

        let mut vec = Self::with_capacity(len as usize);

        for _ in 0..len {
            let item = T::unpack::<_, VERIFY>(unpacker)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

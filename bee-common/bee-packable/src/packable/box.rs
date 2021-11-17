// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::{error::UnpackError, packer::Packer, unpacker::Unpacker, Packable};

use alloc::boxed::Box;
use core::ops::Deref;

impl<T: Packable> Packable for Box<T> {
    type UnpackError = T::UnpackError;

    #[inline(always)]
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.deref().pack(packer)
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(Box::new(T::unpack::<_, VERIFY>(unpacker)?))
    }
}

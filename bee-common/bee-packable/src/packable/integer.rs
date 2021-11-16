// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::convert::Infallible;

macro_rules! impl_packable_for_integer {
    ($ty:ty) => {
        impl Packable for $ty {
            type UnpackError = Infallible;

            #[inline(always)]
            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                packer.pack_bytes(&self.to_le_bytes())
            }

            fn unpack<U: Unpacker, const VERIFY: bool>(
                unpacker: &mut U,
            ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
                let mut bytes = [0u8; core::mem::size_of::<Self>()];
                unpacker.unpack_bytes(&mut bytes)?;
                Ok(Self::from_le_bytes(bytes))
            }
        }
    };
}

impl_packable_for_integer!(u8);
impl_packable_for_integer!(u16);
impl_packable_for_integer!(u32);
impl_packable_for_integer!(u64);
#[cfg(has_u128)]
impl_packable_for_integer!(u128);

/// `usize` integers are packed and unpacked as `u64` integers according to the spec.
impl Packable for usize {
    type UnpackError = Infallible;

    #[inline(always)]
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as u64).pack(packer)
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(u64::unpack::<_, VERIFY>(unpacker).infallible()? as usize)
    }
}

impl_packable_for_integer!(i8);
impl_packable_for_integer!(i16);
impl_packable_for_integer!(i32);
impl_packable_for_integer!(i64);
#[cfg(has_i128)]
impl_packable_for_integer!(i128);

/// `isize` integers are packed and unpacked as `i64` integers according to the spec.
impl Packable for isize {
    type UnpackError = Infallible;

    #[inline(always)]
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as i64).pack(packer)
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(i64::unpack::<_, VERIFY>(unpacker).infallible()? as isize)
    }
}

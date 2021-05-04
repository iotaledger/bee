// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod packer;
mod unpacker;

pub use packer::{PackError, Packer};
pub use unpacker::{UnpackError, Unpacker};

pub trait Packable: Sized {
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error>;
    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error>;
}

macro_rules! impl_packable_for_int {
    ($ty:ty) => {
        impl Packable for $ty {
            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                packer.pack_bytes(&self.to_le_bytes())
            }

            fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
                let bytes: &[u8; std::mem::size_of::<Self>()] = unpacker.unpack_exact_bytes()?;
                Ok(Self::from_le_bytes(*bytes))
            }
        }
    };
}

impl_packable_for_int!(u8);
impl_packable_for_int!(u16);
impl_packable_for_int!(u32);
impl_packable_for_int!(u64);
impl_packable_for_int!(u128);

impl Packable for usize {
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as u64).pack(packer)
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        Ok(u64::unpack(unpacker)? as usize)
    }
}

impl_packable_for_int!(i8);
impl_packable_for_int!(i16);
impl_packable_for_int!(i32);
impl_packable_for_int!(i64);
impl_packable_for_int!(i128);

impl Packable for isize {
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (*self as i64).pack(packer)
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        Ok(u64::unpack(unpacker)? as isize)
    }
}

impl<T: Packable> Packable for Vec<T> {
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.len().pack(packer)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        let len = usize::unpack(unpacker)?;

        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            let item = T::unpack(unpacker)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

impl<T: Packable> Packable for Box<[T]> {
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.len().pack(packer)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        Ok(Vec::<T>::unpack(unpacker)?.into_boxed_slice())
    }
}

impl<T: Packable, const N: usize> Packable for [T; N] {
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        use std::mem::MaybeUninit;

        // Safety: an array of unitialized stuff is initialized.
        let mut array: [MaybeUninit<T>; N] = unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };

        for item in array.iter_mut() {
            let unpacked = T::unpack(unpacker)?;
            // Safety: each `item` is only visited once so we are never overwritting values that
            // are already initialized.
            unsafe {
                item.as_mut_ptr().write(unpacked);
            }
        }

        // Safety: We traversed the whole array and initialized every item.
        Ok(unsafe { (&array as *const [MaybeUninit<T>; N] as *const Self).read() })
    }
}

impl<T: Packable> Packable for Option<T> {
    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match self {
            None => 0u8.pack(packer),
            Some(item) => {
                1u8.pack(packer)?;
                item.pack(packer)
            }
        }
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, U::Error> {
        match u8::unpack(unpacker)? {
            0 => Ok(None),
            1 => Ok(Some(T::unpack(unpacker)?)),
            n => Err(U::Error::invalid_variant(n as u64)),
        }
    }
}

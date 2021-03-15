// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides a `Packable` trait to serialize and deserialize types.

pub use std::io::{Read, Write};

/// A trait to pack and unpack types to and from bytes.
pub trait Packable {
    /// Associated error type.
    type Error: std::fmt::Debug;

    /// Returns the length of the packed bytes.
    fn packed_len(&self) -> usize;

    /// Packs the instance to bytes and writes them to the passed writer.
    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error>;

    /// Packs the instance to bytes and writes them to a newly allocated vector.
    fn pack_new(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.packed_len());
        // Packing to bytes can't fail.
        self.pack(&mut bytes).unwrap();

        bytes
    }

    /// Reads bytes from the passed reader and unpacks them into an instance.
    fn unpack_inner<R: Read + ?Sized, const TRUSTED: bool>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Reads untrusted bytes from the passed reader and unpacks them into an instance.
    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::unpack_inner::<R, false>(reader)
    }
}

impl<const N: usize> Packable for [u8; N] {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        N
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write_all(self)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const TRUSTED: bool>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut bytes = [0u8; N];
        reader.read_exact(&mut bytes)?;

        Ok(bytes)
    }
}

impl Packable for bool {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        (*self as u8).packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (*self as u8).pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const TRUSTED: bool>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(!matches!(u8::unpack_inner::<R, TRUSTED>(reader)?, 0))
    }
}

impl<P> Packable for Vec<P>
where
    P: Packable,
    P::Error: From<std::io::Error>,
{
    type Error = P::Error;

    fn packed_len(&self) -> usize {
        0u64.packed_len() + self.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.len() as u64).pack(writer)?;
        self.iter().try_for_each(|x| x.pack(writer))
    }

    fn unpack_inner<R: Read + ?Sized, const TRUSTED: bool>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        (0..u64::unpack_inner::<R, TRUSTED>(reader)?)
            .map(|_| P::unpack_inner::<R, TRUSTED>(reader))
            .collect()
    }
}

/// Error that occurs on `Option<P: Packable>` operations.
#[derive(Debug)]
pub enum OptionError<E> {
    /// Error that occurs on boolean `Packable` operations.
    Bool(<bool as Packable>::Error),
    /// Error that occurs on inner `Packable` operations.
    Inner(E),
}

impl<E> From<E> for OptionError<E> {
    fn from(inner: E) -> Self {
        OptionError::Inner(inner)
    }
}

impl<P: Packable> Packable for Option<P> {
    type Error = OptionError<P::Error>;

    fn packed_len(&self) -> usize {
        true.packed_len() + self.as_ref().map_or(0, Packable::packed_len)
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        if let Some(p) = self {
            true.pack(writer).map_err(OptionError::Bool)?;
            p.pack(writer).map_err(OptionError::Inner)?;
        } else {
            false.pack(writer).map_err(OptionError::Bool)?;
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const TRUSTED: bool>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(
            if bool::unpack_inner::<R, TRUSTED>(reader).map_err(OptionError::Bool)? {
                Some(P::unpack_inner::<R, TRUSTED>(reader).map_err(OptionError::Inner)?)
            } else {
                None
            },
        )
    }
}

macro_rules! impl_packable_for_num {
    ($ty:ident) => {
        impl Packable for $ty {
            type Error = std::io::Error;

            fn packed_len(&self) -> usize {
                std::mem::size_of_val(&self.to_le_bytes())
            }

            fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
                writer.write_all(&self.to_le_bytes())?;

                Ok(())
            }

            fn unpack_inner<R: Read + ?Sized, const TRUSTED: bool>(reader: &mut R) -> Result<Self, Self::Error>
            where
                Self: Sized,
            {
                let mut bytes = [0; $ty::MIN.to_le_bytes().len()];
                reader.read_exact(&mut bytes)?;

                Ok($ty::from_le_bytes(bytes))
            }
        }
    };
}

impl_packable_for_num!(i8);
impl_packable_for_num!(u8);
impl_packable_for_num!(i16);
impl_packable_for_num!(u16);
impl_packable_for_num!(i32);
impl_packable_for_num!(u32);
impl_packable_for_num!(i64);
impl_packable_for_num!(u64);
#[cfg(has_i128)]
impl_packable_for_num!(i128);
#[cfg(has_u128)]
impl_packable_for_num!(u128);

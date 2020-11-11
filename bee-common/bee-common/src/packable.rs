// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

//! A module that provides a `Packable` trait to serialize and deserialize types.

pub use std::io::{Read, Write};

/// A trait to pack and unpack types to and from bytes.
pub trait Packable {
    /// Associated error type.
    type Error;

    /// Returns the length of the packed bytes.
    fn packed_len(&self) -> usize;

    /// Packs the instance to bytes and writes them to the passed writer.
    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error>;

    /// Packs the instance to bytes and writes them to a newly allocated vector.
    fn pack_new(&self) -> Result<Vec<u8>, Self::Error> {
        let mut bytes = Vec::with_capacity(self.packed_len());
        self.pack(&mut bytes)?;

        Ok(bytes)
    }

    /// Reads bytes from the passed reader and unpacks them into an instance.
    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl Packable for bool {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        (*self as u8).packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (*self as u8).pack(writer)
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(reader)? {
            0 => false,
            _ => true,
        })
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

            fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
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
impl_packable_for_num!(i128);
impl_packable_for_num!(u128);

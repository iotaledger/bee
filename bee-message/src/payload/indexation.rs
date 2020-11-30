// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use digest::Digest;
use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, string::String};
use blake2::Blake2s;

use core::convert::TryInto;

pub const HASHED_INDEX_LENGTH: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Indexation {
    index: String,
    data: Box<[u8]>,
}

impl Indexation {
    pub fn new(index: String, data: &[u8]) -> Result<Self, Error> {
        if index.is_empty() {
            return Err(Error::EmptyIndex);
        }

        Ok(Self {
            index,
            data: data.into(),
        })
    }

    pub fn index(&self) -> &String {
        &self.index
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn hash(&self) -> HashedIndex {
        let mut hasher = Blake2s::new();
        hasher.update(self.index.as_bytes());
        // `Blake2s` output is `HASHED_INDEX_LENGTH` bytes long.
        HashedIndex(hasher.finalize_reset().as_slice().try_into().unwrap())
    }
}

impl Packable for Indexation {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len() + self.index.as_bytes().len() + 0u32.packed_len() + self.data.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.index.as_bytes().len() as u16).pack(writer)?;
        writer.write_all(self.index.as_bytes())?;

        (self.data.len() as u32).pack(writer)?;
        writer.write_all(&self.data)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let index_len = u16::unpack(reader)? as usize;
        let mut index_bytes = vec![0u8; index_len];
        reader.read_exact(&mut index_bytes)?;

        let data_len = u32::unpack(reader)? as usize;
        let mut data_bytes = vec![0u8; data_len];
        reader.read_exact(&mut data_bytes)?;

        Ok(Self {
            index: String::from_utf8(index_bytes)?,
            data: data_bytes.into_boxed_slice(),
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct HashedIndex([u8; HASHED_INDEX_LENGTH]);

// TODO review when we have fixed size index
impl HashedIndex {
    pub fn new(bytes: [u8; HASHED_INDEX_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for HashedIndex {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

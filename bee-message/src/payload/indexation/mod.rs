// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod hash;

pub use hash::{HashedIndex, HASHED_INDEX_LENGTH};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, string::String};
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};

use std::ops::Range;

const INDEX_LENGTH_RANGE: Range<usize> = 1..65;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Indexation {
    index: String,
    data: Box<[u8]>,
}

impl Indexation {
    pub fn new(index: String, data: &[u8]) -> Result<Self, Error> {
        if !INDEX_LENGTH_RANGE.contains(&index.len()) {
            return Err(Error::InvalidIndexLength(index.len()));
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

    // TODO use crypto.rs
    pub fn hash(&self) -> HashedIndex {
        let mut hasher = VarBlake2b::new(HASHED_INDEX_LENGTH).unwrap();

        hasher.update(self.index.as_bytes());

        let mut hash = [0u8; HASHED_INDEX_LENGTH];
        hasher.finalize_variable(|res| hash.copy_from_slice(res));

        HashedIndex::new(hash)
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

        Self::new(String::from_utf8(index_bytes)?, &data_bytes)
    }
}

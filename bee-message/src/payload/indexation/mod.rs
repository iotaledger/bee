// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod hash;

pub use hash::{HashedIndex, HASHED_INDEX_LENGTH};

use crate::{Error, MESSAGE_LENGTH_MAX};

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, string::String};
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};

use std::ops::Range;

pub(crate) const INDEXATION_PAYLOAD_TYPE: u32 = 2;

const INDEX_LENGTH_RANGE: Range<usize> = 1..65;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct IndexationPayload {
    index: String,
    data: Box<[u8]>,
}

impl IndexationPayload {
    pub fn new(index: String, data: &[u8]) -> Result<Self, Error> {
        if !INDEX_LENGTH_RANGE.contains(&index.len()) {
            return Err(Error::InvalidIndexationLength(index.len()));
        }

        if data.len() > MESSAGE_LENGTH_MAX {
            return Err(Error::InvalidIndexationLength(data.len()));
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

impl Packable for IndexationPayload {
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let index_len = u16::unpack(reader)? as usize;

        if !INDEX_LENGTH_RANGE.contains(&index_len) {
            return Err(Error::InvalidIndexationLength(index_len));
        }

        let mut index_bytes = vec![0u8; index_len];
        reader.read_exact(&mut index_bytes)?;

        let data_len = u32::unpack(reader)? as usize;

        if data_len > MESSAGE_LENGTH_MAX {
            return Err(Error::InvalidIndexationLength(data_len));
        }

        let mut data_bytes = vec![0u8; data_len];
        reader.read_exact(&mut data_bytes)?;

        Ok(Self {
            index: String::from_utf8(index_bytes)?,
            data: data_bytes.into_boxed_slice(),
        })
    }
}

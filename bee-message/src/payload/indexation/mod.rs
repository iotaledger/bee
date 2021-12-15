// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the indexation payload.

mod padded;

use crate::{Error, Message};

pub use padded::PaddedIndex;

use bee_common::packable::{Packable, Read, Write};

use alloc::boxed::Box;
use core::ops::RangeInclusive;

/// A payload which holds an index and associated data.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexationPayload {
    index: Box<[u8]>,
    data: Box<[u8]>,
}

impl IndexationPayload {
    /// The payload kind of an `IndexationPayload`.
    pub const KIND: u32 = 2;
    /// Valid lengths for an indexation payload index.
    pub const LENGTH_RANGE: RangeInclusive<usize> = 1..=PaddedIndex::LENGTH;

    /// Creates a new `IndexationPayload`.
    pub fn new(index: &[u8], data: &[u8]) -> Result<Self, Error> {
        if !IndexationPayload::LENGTH_RANGE.contains(&index.len()) {
            return Err(Error::InvalidIndexationIndexLength(index.len()));
        }

        if data.len() > Message::LENGTH_MAX {
            return Err(Error::InvalidIndexationDataLength(data.len()));
        }

        Ok(Self {
            index: index.into(),
            data: data.into(),
        })
    }

    /// Returns the index of an `IndexationPayload`.
    pub fn index(&self) -> &[u8] {
        &self.index
    }

    /// Returns the padded index of an `IndexationPayload`.
    pub fn padded_index(&self) -> PaddedIndex {
        let mut padded_index = [0u8; PaddedIndex::LENGTH];
        padded_index[..self.index.len()].copy_from_slice(&self.index);
        PaddedIndex::from(padded_index)
    }

    /// Returns the data of an `IndexationPayload`.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Packable for IndexationPayload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len() + self.index.len() + 0u32.packed_len() + self.data.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.index.len() as u16).pack(writer)?;
        writer.write_all(&self.index)?;

        (self.data.len() as u32).pack(writer)?;
        writer.write_all(&self.data)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let index_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK && !IndexationPayload::LENGTH_RANGE.contains(&index_len) {
            return Err(Error::InvalidIndexationIndexLength(index_len));
        }

        let mut index = vec![0u8; index_len];
        reader.read_exact(&mut index)?;

        let data_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK && data_len > Message::LENGTH_MAX {
            return Err(Error::InvalidIndexationDataLength(data_len));
        }

        let mut data = vec![0u8; data_len];
        reader.read_exact(&mut data)?;

        Ok(Self {
            index: index.into(),
            data: data.into(),
        })
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::Output, spent::Spent, Error};

use bee_common::packable::{Packable, Read, Write};

pub(crate) struct MilestoneDiff {
    index: u32,
    created: Vec<Output>,
    consumed: Vec<Spent>,
}

impl Packable for MilestoneDiff {
    type Error = Error;

    fn packed_len(&self) -> usize {
        // TODO finish
        self.index.packed_len() + 0u64.packed_len() + 0u64.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;
        (self.created.len() as u64).pack(writer)?;
        for output in self.created.iter() {
            output.pack(writer)?;
        }
        (self.consumed.len() as u64).pack(writer)?;
        for spent in self.consumed.iter() {
            spent.pack(writer)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let index = u32::unpack(reader)?;
        let created_count = u64::unpack(reader)? as usize;
        let mut created = Vec::with_capacity(created_count);
        for _ in 0..created_count {
            created.push(Output::unpack(reader)?);
        }
        let consumed_count = u64::unpack(reader)? as usize;
        let mut consumed = Vec::with_capacity(consumed_count);
        for _ in 0..consumed_count {
            consumed.push(Spent::unpack(reader)?);
        }

        Ok(Self {
            index,
            created,
            consumed,
        })
    }
}

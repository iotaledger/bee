// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::{Packable, Read, Write};
use bee_protocol::MilestoneIndex;

#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LedgerIndex(MilestoneIndex);

impl From<MilestoneIndex> for LedgerIndex {
    fn from(index: MilestoneIndex) -> Self {
        Self(index)
    }
}

impl LedgerIndex {
    pub fn new(index: MilestoneIndex) -> Self {
        index.into()
    }
}

impl Packable for LedgerIndex {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self(MilestoneIndex::unpack(reader)?))
    }
}

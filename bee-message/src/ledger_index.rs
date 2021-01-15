// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::milestone::MilestoneIndex;

use bee_common::packable::{Packable, Read, Write};

use std::ops::Deref;

#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LedgerIndex(pub MilestoneIndex);

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

impl Deref for LedgerIndex {
    type Target = <MilestoneIndex as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.0
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(MilestoneIndex::unpack(reader)?))
    }
}

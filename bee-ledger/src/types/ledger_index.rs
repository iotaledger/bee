// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::{Packable, Read, Write};
use bee_message::milestone::MilestoneIndex;

use core::ops::Deref;

#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LedgerIndex(pub MilestoneIndex);

impl LedgerIndex {
    pub fn new(index: MilestoneIndex) -> Self {
        index.into()
    }
}

impl From<MilestoneIndex> for LedgerIndex {
    fn from(index: MilestoneIndex) -> Self {
        Self(index)
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

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(MilestoneIndex::unpack_inner::<R, CHECK>(reader)?))
    }
}

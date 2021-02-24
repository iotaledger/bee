// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::milestone::MilestoneIndex;

#[derive(Debug)]
pub struct ReceiptKey {
    migrated_at: MilestoneIndex,
    // TODO needed ?
    included_in: MilestoneIndex,
}

impl ReceiptKey {
    pub fn new(migrated_at: MilestoneIndex, included_in: MilestoneIndex) -> Self {
        Self {
            migrated_at,
            included_in,
        }
    }

    pub fn migrated_at(&self) -> &MilestoneIndex {
        &self.migrated_at
    }

    pub fn included_in(&self) -> &MilestoneIndex {
        &self.included_in
    }
}

impl Packable for ReceiptKey {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.migrated_at.packed_len() + self.included_in.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.migrated_at.pack(writer)?;
        self.included_in.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(
            MilestoneIndex::unpack(reader)?,
            MilestoneIndex::unpack(reader)?,
        ))
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, milestone::MilestoneIndex, payload::transaction::TransactionId};

use bee_common::packable::{Packable, Read, Write};

#[derive(Debug)]
pub struct ConsumedOutput {
    target: TransactionId,
    index: MilestoneIndex,
}

impl ConsumedOutput {
    pub fn new(target: TransactionId, index: MilestoneIndex) -> Self {
        Self { target, index }
    }

    pub fn target(&self) -> &TransactionId {
        &self.target
    }

    pub fn index(&self) -> MilestoneIndex {
        self.index
    }
}

impl Packable for ConsumedOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.target.packed_len() + self.index.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.target.pack(writer)?;
        self.index.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self {
            target: TransactionId::unpack(reader)?,
            index: MilestoneIndex::unpack(reader)?,
        })
    }
}

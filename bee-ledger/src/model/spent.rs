// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::{milestone::MilestoneIndex, payload::transaction::TransactionId};

#[derive(Debug)]
pub struct Spent {
    target: TransactionId,
    index: MilestoneIndex,
}

impl Spent {
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

impl Packable for Spent {
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

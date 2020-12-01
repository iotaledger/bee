// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::payload::transaction::TransactionId;
use bee_protocol::MilestoneIndex;

pub struct Spent {
    target: TransactionId,
    index: MilestoneIndex,
}

impl Spent {
    pub fn new(target: TransactionId, index: MilestoneIndex) -> Self {
        Self { target, index }
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let target = TransactionId::unpack(reader)?;
        let index = MilestoneIndex::unpack(reader)?;

        Ok(Self { target, index })
    }
}

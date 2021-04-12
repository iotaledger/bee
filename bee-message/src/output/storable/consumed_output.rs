// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, milestone::MilestoneIndex, payload::transaction::TransactionId};

use bee_common::packable::{Packable, Read, Write};

#[derive(Clone, Debug)]
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

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let target = TransactionId::unpack_inner::<R, CHECK>(reader)?;
        let index = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self { target, index })
    }
}

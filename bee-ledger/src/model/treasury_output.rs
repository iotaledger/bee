// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO here or in bee-message ?

use crate::model::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::payload::{milestone::MilestoneId, transaction};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryOutput {
    inner: transaction::TreasuryOutput,
    milestone_id: MilestoneId,
}

impl TreasuryOutput {
    pub fn new(inner: transaction::TreasuryOutput, milestone_id: MilestoneId) -> Self {
        Self { inner, milestone_id }
    }

    pub fn inner(&self) -> &transaction::TreasuryOutput {
        &self.inner
    }

    pub fn milestone_id(&self) -> &MilestoneId {
        &self.milestone_id
    }
}

impl Packable for TreasuryOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.inner.packed_len() + self.inner.packed_len() + self.milestone_id.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.inner.pack(writer)?;
        self.milestone_id.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(
            transaction::TreasuryOutput::unpack(reader)?,
            MilestoneId::unpack(reader)?,
        ))
    }
}

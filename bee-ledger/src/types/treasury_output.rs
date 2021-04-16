// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::error::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::{output, payload::milestone::MilestoneId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryOutput {
    inner: output::TreasuryOutput,
    milestone_id: MilestoneId,
}

impl TreasuryOutput {
    pub fn new(inner: output::TreasuryOutput, milestone_id: MilestoneId) -> Self {
        Self { inner, milestone_id }
    }

    pub fn inner(&self) -> &output::TreasuryOutput {
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

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let inner = output::TreasuryOutput::unpack_inner::<R, CHECK>(reader)?;
        let milestone_id = MilestoneId::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self::new(inner, milestone_id))
    }
}

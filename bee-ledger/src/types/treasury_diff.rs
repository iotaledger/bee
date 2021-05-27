// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::error::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::payload::milestone::MilestoneId;

/// Wraps together the identifiers of the milestones that created and consumed treasury outputs.
#[derive(Debug)]
pub struct TreasuryDiff {
    created: MilestoneId,
    consumed: MilestoneId,
}

impl TreasuryDiff {
    /// Creates a new `TreasuryDiff`.
    pub fn new(created: MilestoneId, consumed: MilestoneId) -> Self {
        Self { created, consumed }
    }

    /// Returns the id of the milestone that created the treasury output associated to the `TreasuryDiff`.
    pub fn created(&self) -> &MilestoneId {
        &self.created
    }

    /// Returns the id of the milestone that consumed the treasury input associated to the `TreasuryDiff`.
    pub fn consumed(&self) -> &MilestoneId {
        &self.consumed
    }
}

impl Packable for TreasuryDiff {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.created.packed_len() + self.consumed.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.created.pack(writer)?;
        self.consumed.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let created = MilestoneId::unpack_inner::<R, CHECK>(reader)?;
        let consumed = MilestoneId::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self { created, consumed })
    }
}

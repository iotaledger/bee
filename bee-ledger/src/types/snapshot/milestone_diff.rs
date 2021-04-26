// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{ConsumedOutput, CreatedOutput, Error};

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
    output::{Output, OutputId, TreasuryOutput},
    payload::{
        milestone::{MilestoneId, MilestonePayload},
        transaction::TransactionId,
        Payload,
    },
    MessageId,
};

use std::collections::HashMap;

/// Describe the ledger changes occurring within a milestone.
pub struct MilestoneDiff {
    milestone: MilestonePayload,
    consumed_treasury: Option<(TreasuryOutput, MilestoneId)>,
    created_outputs: HashMap<OutputId, CreatedOutput>,
    consumed_outputs: HashMap<OutputId, (CreatedOutput, ConsumedOutput)>,
}

impl MilestoneDiff {
    /// Returns the milestone of a `MilestoneDiff`.
    pub fn milestone(&self) -> &MilestonePayload {
        &self.milestone
    }

    /// Returns the consumed treasury of a `MilestoneDiff`.
    pub fn consumed_treasury(&self) -> Option<&(TreasuryOutput, MilestoneId)> {
        self.consumed_treasury.as_ref()
    }

    /// Returns the created outputs of a `MilestoneDiff`.
    pub fn created(&self) -> &HashMap<OutputId, CreatedOutput> {
        &self.created_outputs
    }

    /// Returns the consumed outputs of a `MilestoneDiff`.
    pub fn consumed(&self) -> &HashMap<OutputId, (CreatedOutput, ConsumedOutput)> {
        &self.consumed_outputs
    }
}

impl Packable for MilestoneDiff {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u32.packed_len()
            + std::mem::size_of_val(&MilestonePayload::KIND)
            + self.milestone.packed_len()
            + if let Some((treasury_output, milestone_id)) = self.consumed_treasury.as_ref() {
                milestone_id.packed_len() + treasury_output.packed_len()
            } else {
                0
            }
            + 0u64.packed_len()
            + self.created_outputs.iter().fold(0, |acc, (output_id, created_output)| {
                acc + output_id.packed_len() + created_output.packed_len()
            })
            + 0u64.packed_len()
            + self
                .consumed_outputs
                .iter()
                .fold(0, |acc, (output_id, (created_output, consumed_output))| {
                    acc + output_id.packed_len() + created_output.packed_len() + consumed_output.target().packed_len()
                })
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.milestone.packed_len() as u32 + MilestonePayload::KIND).pack(writer)?;
        MilestonePayload::KIND.pack(writer)?;
        self.milestone.pack(writer)?;

        if self.milestone.essence().receipt().is_some() {
            if let Some((treasury_output, milestone_id)) = self.consumed_treasury.as_ref() {
                milestone_id.pack(writer)?;
                treasury_output.pack(writer)?;
            } else {
                return Err(Error::MissingConsumedTreasury);
            }
        }

        (self.created_outputs.len() as u64).pack(writer)?;
        for (output_id, created) in self.created_outputs.iter() {
            created.message_id().pack(writer)?;
            output_id.pack(writer)?;
            created.pack(writer)?;
        }

        (self.consumed_outputs.len() as u64).pack(writer)?;
        for (output_id, (created, consumed)) in self.consumed_outputs.iter() {
            created.message_id().pack(writer)?;
            output_id.pack(writer)?;
            created.inner().pack(writer)?;
            consumed.target().pack(writer)?;
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let milestone_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;
        let milestone = match Payload::unpack_inner::<R, CHECK>(reader)? {
            Payload::Milestone(milestone) => milestone,
            payload => return Err(Error::InvalidPayloadKind(payload.kind())),
        };

        if milestone_len != milestone.packed_len() + std::mem::size_of_val(&MilestonePayload::KIND) {
            return Err(Error::MilestoneLengthMismatch(
                milestone_len,
                milestone.packed_len() + std::mem::size_of_val(&MilestonePayload::KIND),
            ));
        }

        let consumed_treasury = if milestone.essence().receipt().is_some() {
            let milestone_id = MilestoneId::unpack_inner::<R, CHECK>(reader)?;
            let amount = u64::unpack_inner::<R, CHECK>(reader)?;

            Some((TreasuryOutput::new(amount)?, milestone_id))
        } else {
            None
        };

        let created_count = u64::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut created_outputs = HashMap::with_capacity(created_count);

        for _ in 0..created_count {
            let message_id = MessageId::unpack_inner::<R, CHECK>(reader)?;
            let output_id = OutputId::unpack_inner::<R, CHECK>(reader)?;
            let output = Output::unpack_inner::<R, CHECK>(reader)?;

            created_outputs.insert(output_id, CreatedOutput::new(message_id, output));
        }

        let consumed_count = u64::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut consumed_outputs = HashMap::with_capacity(consumed_count);

        for _ in 0..consumed_count {
            let message_id = MessageId::unpack_inner::<R, CHECK>(reader)?;
            let output_id = OutputId::unpack_inner::<R, CHECK>(reader)?;
            let output = Output::unpack_inner::<R, CHECK>(reader)?;
            let target = TransactionId::unpack_inner::<R, CHECK>(reader)?;

            consumed_outputs.insert(
                output_id,
                (
                    CreatedOutput::new(message_id, output),
                    ConsumedOutput::new(target, milestone.essence().index()),
                ),
            );
        }

        Ok(Self {
            milestone: *milestone,
            created_outputs,
            consumed_outputs,
            consumed_treasury,
        })
    }
}

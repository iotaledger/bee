// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO review this file

use crate::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
    output::{ConsumedOutput, CreatedOutput, Output, OutputId, TreasuryOutput},
    payload::{
        milestone::{MilestoneId, MilestonePayload},
        transaction::TransactionId,
        Payload,
    },
    MessageId,
};

use std::collections::HashMap;

pub struct MilestoneDiff {
    milestone: MilestonePayload,
    created: HashMap<OutputId, CreatedOutput>,
    consumed: HashMap<OutputId, (CreatedOutput, ConsumedOutput)>,
    consumed_treasury: Option<(TreasuryOutput, MilestoneId)>,
}

impl MilestoneDiff {
    pub fn milestone(&self) -> &MilestonePayload {
        &self.milestone
    }

    pub fn created(&self) -> &HashMap<OutputId, CreatedOutput> {
        &self.created
    }

    pub fn consumed(&self) -> &HashMap<OutputId, (CreatedOutput, ConsumedOutput)> {
        &self.consumed
    }

    pub fn consumed_treasury(&self) -> Option<&(TreasuryOutput, MilestoneId)> {
        self.consumed_treasury.as_ref()
    }
}

impl Packable for MilestoneDiff {
    type Error = Error;

    fn packed_len(&self) -> usize {
        // TODO finish
        self.milestone.packed_len() + 0u64.packed_len() + 0u64.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        // TODO finish
        self.milestone.pack(writer)?;

        (self.created.len() as u64).pack(writer)?;
        for (output_id, output) in self.created.iter() {
            output.message_id().pack(writer)?;
            output_id.pack(writer)?;
            output.pack(writer)?;
        }

        (self.consumed.len() as u64).pack(writer)?;
        for (_output_id, _spent) in self.consumed.iter() {
            // TODO finish
            // spent.pack(writer)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let milestone_len = u32::unpack(reader)? as usize;
        let milestone = match Payload::unpack(reader)? {
            Payload::Milestone(milestone) => milestone,
            _ => return Err(Error::InvalidPayloadKind),
        };

        // TODO
        if milestone_len != milestone.packed_len() {}

        let consumed_treasury = if milestone.essence().receipt().is_some() {
            let milestone_id = MilestoneId::unpack(reader)?;
            let amount = u64::unpack(reader)?;
            Some((TreasuryOutput::new(amount)?, milestone_id))
        } else {
            None
        };

        let created_count = u64::unpack(reader)? as usize;
        let mut created = HashMap::new();

        for _ in 0..created_count {
            let message_id = MessageId::unpack(reader)?;
            let output_id = OutputId::unpack(reader)?;
            let output = Output::unpack(reader)?;

            created.insert(output_id, CreatedOutput::new(message_id, output));
        }

        let consumed_count = u64::unpack(reader)? as usize;
        let mut consumed = HashMap::new();

        for _ in 0..consumed_count {
            let message_id = MessageId::unpack(reader)?;
            let output_id = OutputId::unpack(reader)?;
            let output = Output::unpack(reader)?;
            let target = TransactionId::unpack(reader)?;

            consumed.insert(
                output_id,
                (
                    CreatedOutput::new(message_id, output),
                    ConsumedOutput::new(target, milestone.essence().index()),
                ),
            );
        }

        Ok(Self {
            milestone: *milestone,
            created,
            consumed,
            consumed_treasury,
        })
    }
}

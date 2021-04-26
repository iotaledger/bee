// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO review this file

use crate::{
    snapshot::error::Error,
    types::{ConsumedOutput, CreatedOutput},
};

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
        (self.milestone.packed_len() as u32 + MilestonePayload::KIND).pack(writer)?;
        MilestonePayload::KIND.pack(writer)?;
        self.milestone.pack(writer)?;

        if self.milestone.essence().receipt().is_some() {
            // TODO unwrap
            self.consumed_treasury.as_ref().unwrap().1.pack(writer)?;
            self.consumed_treasury.as_ref().unwrap().0.pack(writer)?;
        }

        (self.created.len() as u64).pack(writer)?;
        for (output_id, created) in self.created.iter() {
            created.message_id().pack(writer)?;
            output_id.pack(writer)?;
            created.pack(writer)?;
        }

        (self.consumed.len() as u64).pack(writer)?;
        for (output_id, (created, consumed)) in self.consumed.iter() {
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

        if milestone_len != milestone.packed_len() + MilestonePayload::KIND as usize {
            // TODO
        }

        let consumed_treasury = if milestone.essence().receipt().is_some() {
            let milestone_id = MilestoneId::unpack_inner::<R, CHECK>(reader)?;
            let amount = u64::unpack_inner::<R, CHECK>(reader)?;

            Some((TreasuryOutput::new(amount)?, milestone_id))
        } else {
            None
        };

        let created_count = u64::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut created = HashMap::with_capacity(created_count);

        for _ in 0..created_count {
            let message_id = MessageId::unpack_inner::<R, CHECK>(reader)?;
            let output_id = OutputId::unpack_inner::<R, CHECK>(reader)?;
            let output = Output::unpack_inner::<R, CHECK>(reader)?;

            created.insert(output_id, CreatedOutput::new(message_id, output));
        }

        let consumed_count = u64::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut consumed = HashMap::with_capacity(consumed_count);

        for _ in 0..consumed_count {
            let message_id = MessageId::unpack_inner::<R, CHECK>(reader)?;
            let output_id = OutputId::unpack_inner::<R, CHECK>(reader)?;
            let output = Output::unpack_inner::<R, CHECK>(reader)?;
            let target = TransactionId::unpack_inner::<R, CHECK>(reader)?;

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

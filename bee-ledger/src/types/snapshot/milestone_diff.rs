// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{ConsumedOutput, CreatedOutput, Error};

use bee_message::{
    output::{Output, OutputId, TreasuryOutput},
    payload::{
        milestone::{MilestoneId, MilestonePayload},
        transaction::TransactionId,
        Payload,
    },
    MessageId,
};
use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
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
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (self.milestone.packed_len() as u32 + MilestonePayload::KIND).pack(packer)?;
        MilestonePayload::KIND.pack(packer)?;
        self.milestone.pack(packer)?;

        if self.milestone.essence().receipt().is_some() {
            // The current `unpack` implementation guarantees that `consumed_treasury` is some if
            // the `receipt` is some and that is the only way to create a `MilestoneDiff`.
            if let Some((treasury_output, milestone_id)) = self.consumed_treasury.as_ref() {
                milestone_id.pack(packer)?;
                treasury_output.pack(packer)?;
            }
        }

        (self.created_outputs.len() as u64).pack(packer)?;
        for (output_id, created) in self.created_outputs.iter() {
            created.message_id().pack(packer)?;
            output_id.pack(packer)?;
            created.pack(packer)?;
        }

        (self.consumed_outputs.len() as u64).pack(packer)?;
        for (output_id, (created, consumed)) in self.consumed_outputs.iter() {
            created.message_id().pack(packer)?;
            output_id.pack(packer)?;
            created.inner().pack(packer)?;
            consumed.target().pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let milestone_len = u32::unpack::<_, VERIFY>(unpacker).infallible()? as usize;
        let payload = Payload::unpack::<_, VERIFY>(unpacker).coerce()?;
        let milestone = match payload {
            Payload::Milestone(milestone) => milestone,
            Payload::Indexation(_)
            | Payload::Receipt(_)
            | Payload::Transaction(_)
            | Payload::TreasuryTransaction(_) => {
                return Err(UnpackError::Packable(Error::InvalidPayloadKind(payload.kind())));
            }
        };

        if milestone_len != milestone.packed_len() + std::mem::size_of_val(&MilestonePayload::KIND) {
            return Err(UnpackError::Packable(Error::MilestoneLengthMismatch(
                milestone_len,
                milestone.packed_len() + std::mem::size_of_val(&MilestonePayload::KIND),
            )));
        }

        let consumed_treasury = if milestone.essence().receipt().is_some() {
            let milestone_id = MilestoneId::unpack::<_, VERIFY>(unpacker).infallible()?;
            let amount = u64::unpack::<_, VERIFY>(unpacker).infallible()?;

            Some((
                TreasuryOutput::new(amount).map_err(UnpackError::from_packable)?,
                milestone_id,
            ))
        } else {
            None
        };

        let created_count = u64::unpack::<_, VERIFY>(unpacker).infallible()? as usize;
        let mut created_outputs = HashMap::with_capacity(created_count);

        for _ in 0..created_count {
            let message_id = MessageId::unpack::<_, VERIFY>(unpacker).infallible()?;
            let output_id = OutputId::unpack::<_, VERIFY>(unpacker).coerce()?;
            let output = Output::unpack::<_, VERIFY>(unpacker).coerce()?;

            created_outputs.insert(output_id, CreatedOutput::new(message_id, output));
        }

        let consumed_count = u64::unpack::<_, VERIFY>(unpacker).infallible()? as usize;
        let mut consumed_outputs = HashMap::with_capacity(consumed_count);

        for _ in 0..consumed_count {
            let message_id = MessageId::unpack::<_, VERIFY>(unpacker).infallible()?;
            let output_id = OutputId::unpack::<_, VERIFY>(unpacker).coerce()?;
            let output = Output::unpack::<_, VERIFY>(unpacker).coerce()?;
            let target = TransactionId::unpack::<_, VERIFY>(unpacker).infallible()?;

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

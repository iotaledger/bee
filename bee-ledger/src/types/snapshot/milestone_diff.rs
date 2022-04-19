// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_message::{
    output::{OutputId, TreasuryOutput},
    payload::{
        milestone::{MilestoneId, MilestonePayload},
        transaction::TransactionId,
        Payload,
    },
};
use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use crate::types::{ConsumedOutput, CreatedOutput, Error};

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
            if let Some((treasury_output, milestone_id)) = self.consumed_treasury.as_ref() {
                milestone_id.pack(packer)?;
                treasury_output.pack(packer)?;
            } else {
                // This never happens because `MilestoneDiff` values can only be created by unpacking them and the
                // `unpack` implementation guarantees that the `consumed_treasury` field is some if the receipt is some.
                unreachable!()
            }
        }

        (self.created_outputs.len() as u64).pack(packer)?;

        for (output_id, created) in self.created_outputs.iter() {
            output_id.pack(packer)?;
            created.pack(packer)?;
        }

        (self.consumed_outputs.len() as u64).pack(packer)?;

        for (output_id, (created, consumed)) in self.consumed_outputs.iter() {
            output_id.pack(packer)?;
            created.pack(packer)?;
            consumed.target().pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let milestone_len = u32::unpack::<_, VERIFY>(unpacker).coerce()? as usize;
        let payload = Payload::unpack::<_, VERIFY>(unpacker).coerce()?;
        let milestone = match payload {
            Payload::Milestone(milestone) => milestone,
            _ => {
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
            let milestone_id = MilestoneId::unpack::<_, VERIFY>(unpacker).coerce()?;
            let amount = u64::unpack::<_, VERIFY>(unpacker).coerce()?;

            Some((
                TreasuryOutput::new(amount).map_err(UnpackError::from_packable)?,
                milestone_id,
            ))
        } else {
            None
        };

        let created_count = u64::unpack::<_, VERIFY>(unpacker).coerce()?;
        let mut created_outputs = HashMap::with_capacity(created_count as usize);

        for _ in 0..created_count {
            let output_id = OutputId::unpack::<_, VERIFY>(unpacker).coerce()?;
            let created_output = CreatedOutput::unpack::<_, VERIFY>(unpacker).coerce()?;

            created_outputs.insert(output_id, created_output);
        }

        let consumed_count = u64::unpack::<_, VERIFY>(unpacker).coerce()?;
        let mut consumed_outputs = HashMap::with_capacity(consumed_count as usize);

        for _ in 0..consumed_count {
            let output_id = OutputId::unpack::<_, VERIFY>(unpacker).coerce()?;
            let created_output = CreatedOutput::unpack::<_, VERIFY>(unpacker).coerce()?;
            let target = TransactionId::unpack::<_, VERIFY>(unpacker).coerce()?;

            consumed_outputs.insert(
                output_id,
                (
                    created_output,
                    ConsumedOutput::new(
                        target,
                        milestone.essence().index(),
                        milestone.essence().timestamp() as u32,
                    ),
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

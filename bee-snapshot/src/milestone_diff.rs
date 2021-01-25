// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO review this file

use crate::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
    milestone::MilestoneIndex,
    payload::transaction::{ConsumedOutput, CreatedOutput, Output, OutputId, TransactionId},
    MessageId,
};

use std::collections::HashMap;

pub struct MilestoneDiff {
    index: MilestoneIndex,
    created: HashMap<OutputId, CreatedOutput>,
    consumed: HashMap<OutputId, ConsumedOutput>,
}

impl MilestoneDiff {
    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    pub fn created(&self) -> &HashMap<OutputId, CreatedOutput> {
        &self.created
    }

    pub fn consumed(&self) -> &HashMap<OutputId, ConsumedOutput> {
        &self.consumed
    }
}

impl Packable for MilestoneDiff {
    type Error = Error;

    fn packed_len(&self) -> usize {
        // TODO finish
        self.index.packed_len() + 0u64.packed_len() + 0u64.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;

        (self.created.len() as u64).pack(writer)?;
        for (output_id, output) in self.created.iter() {
            output.message_id().pack(writer)?;
            output_id.pack(writer)?;
            output.pack(writer)?;
        }

        (self.consumed.len() as u64).pack(writer)?;
        for (_output_id, spent) in self.consumed.iter() {
            // TODO finish
            spent.pack(writer)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = MilestoneIndex::unpack(reader)?;

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
            let _message_id = MessageId::unpack(reader)?;
            let output_id = OutputId::unpack(reader)?;
            let _output = Output::unpack(reader)?;
            let _target = TransactionId::unpack(reader)?;
            consumed.insert(output_id, ConsumedOutput::new(*output_id.transaction_id(), index));
        }

        Ok(Self {
            index,
            created,
            consumed,
        })
    }
}

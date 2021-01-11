// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::{
        transaction::{
            constants::{INPUT_OUTPUT_COUNT_RANGE, INPUT_OUTPUT_INDEX_RANGE, IOTA_SUPPLY},
            input::Input,
            output::Output,
        },
        Payload,
    },
    Error,
};

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, vec::Vec};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TransactionEssence {
    inputs: Box<[Input]>,
    outputs: Box<[Output]>,
    payload: Option<Payload>,
}

impl TransactionEssence {
    pub fn builder() -> TransactionEssenceBuilder {
        TransactionEssenceBuilder::new()
    }

    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }
}

impl Packable for TransactionEssence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + 0u16.packed_len()
            + self.inputs.iter().map(|input| input.packed_len()).sum::<usize>()
            + 0u16.packed_len()
            + self.outputs.iter().map(|output| output.packed_len()).sum::<usize>()
            + 0u32.packed_len()
            + self.payload.iter().map(|payload| payload.packed_len()).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        0u8.pack(writer)?;

        (self.inputs.len() as u16).pack(writer)?;
        for input in self.inputs.iter() {
            input.pack(writer)?;
        }

        (self.outputs.len() as u16).pack(writer)?;
        for output in self.outputs.iter() {
            output.pack(writer)?;
        }

        match self.payload {
            Some(ref payload) => {
                (payload.packed_len() as u32).pack(writer)?;
                payload.pack(writer)?;
            }
            None => 0u32.pack(writer)?,
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let essence_type = u8::unpack(reader)?;

        if essence_type != 0u8 {
            return Err(Self::Error::InvalidType(0, essence_type));
        }

        let inputs_len = u16::unpack(reader)? as usize;
        let mut inputs = Vec::with_capacity(inputs_len);
        for _ in 0..inputs_len {
            inputs.push(Input::unpack(reader)?);
        }

        let outputs_len = u16::unpack(reader)? as usize;
        let mut outputs = Vec::with_capacity(outputs_len);
        for _ in 0..outputs_len {
            outputs.push(Output::unpack(reader)?);
        }

        let mut builder = Self::builder().with_inputs(inputs).with_outputs(outputs);

        let payload_len = u32::unpack(reader)? as usize;
        if payload_len > 0 {
            let payload = Payload::unpack(reader)?;
            if payload_len != payload.packed_len() {
                return Err(Self::Error::InvalidAnnouncedLength(payload_len, payload.packed_len()));
            }
            builder = builder.with_payload(payload);
        }

        builder.finish()
    }
}

#[derive(Debug, Default)]
pub struct TransactionEssenceBuilder {
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,
    pub(crate) payload: Option<Payload>,
}

impl TransactionEssenceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_inputs(mut self, inputs: Vec<Input>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn add_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<Output>) -> Self {
        self.outputs = outputs;
        self
    }

    pub fn add_output(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn finish(mut self) -> Result<TransactionEssence, Error> {
        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.inputs.len()) {
            return Err(Error::InvalidInputOutputCount(self.inputs.len()));
        }

        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.outputs.len()) {
            return Err(Error::InvalidInputOutputCount(self.outputs.len()));
        }

        if !matches!(self.payload, None | Some(Payload::Indexation(_))) {
            return Err(Error::InvalidTransactionPayload);
        }

        // Inputs validation

        for input in self.inputs.iter() {
            match input {
                Input::UTXO(u) => {
                    // Transaction Output Index must be 0 ≤ x < 127
                    if !INPUT_OUTPUT_INDEX_RANGE.contains(&u.output_id().index()) {
                        return Err(Error::InvalidInputOutputIndex(u.output_id().index()));
                    }

                    // Every combination of Transaction ID + Transaction Output Index must be unique in the inputs set.
                    if self.inputs.iter().filter(|i| *i == input).count() > 1 {
                        return Err(Error::DuplicateError);
                    }
                }
            }
        }

        // Outputs validation

        let mut total: u64 = 0;

        for output in self.outputs.iter() {
            match output {
                Output::SignatureLockedSingle(u) => {
                    // The Address must be unique in the set of SigLockedSingleDeposits.
                    if self
                        .outputs
                        .iter()
                        .filter(|j| match *j {
                            Output::SignatureLockedSingle(s) => s.address() == u.address(),
                        })
                        .count()
                        > 1
                    {
                        return Err(Error::DuplicateError);
                    }

                    total = total
                        .checked_add(u.amount())
                        .ok_or(Error::InvalidAccumulatedOutput((total + u.amount()) as u128))?;
                }
            }
            // Accumulated output balance must not exceed the total supply of tokens.
            if total > IOTA_SUPPLY {
                return Err(Error::InvalidAccumulatedOutput(total as u128));
            }
        }

        self.inputs.sort();
        self.outputs.sort();

        Ok(TransactionEssence {
            inputs: self.inputs.into_boxed_slice(),
            outputs: self.outputs.into_boxed_slice(),
            payload: self.payload,
        })
    }
}

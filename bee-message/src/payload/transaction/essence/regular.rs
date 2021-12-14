// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constant::IOTA_SUPPLY,
    input::{Input, INPUT_COUNT_RANGE},
    output::{Output, OUTPUT_COUNT_RANGE},
    payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, Payload},
    Error,
};

use bee_common::packable::{Packable, Read, Write};

use alloc::{boxed::Box, vec::Vec};

/// A builder to build a [`RegularTransactionEssence`].
#[derive(Debug, Default)]
pub struct RegularTransactionEssenceBuilder {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    payload: Option<Payload>,
}

impl RegularTransactionEssenceBuilder {
    /// Creates a new [`RegularTransactionEssenceBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds inputs to a [`RegularTransactionEssenceBuilder`]
    pub fn with_inputs(mut self, inputs: Vec<Input>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Add an input to a [`RegularTransactionEssenceBuilder`].
    pub fn add_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    /// Add outputs to a [`RegularTransactionEssenceBuilder`].
    pub fn with_outputs(mut self, outputs: Vec<Output>) -> Self {
        self.outputs = outputs;
        self
    }

    /// Add an output to a [`RegularTransactionEssenceBuilder`].
    pub fn add_output(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }

    /// Add a payload to a [`RegularTransactionEssenceBuilder`].
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Finishes a [`RegularTransactionEssenceBuilder`] into a [`RegularTransactionEssence`].
    pub fn finish(self) -> Result<RegularTransactionEssence, Error> {
        if !INPUT_COUNT_RANGE.contains(&(self.inputs.len() as u16)) {
            return Err(Error::InvalidInputOutputCount(self.inputs.len() as u16));
        }

        if !OUTPUT_COUNT_RANGE.contains(&(self.outputs.len() as u16)) {
            return Err(Error::InvalidInputOutputCount(self.outputs.len() as u16));
        }

        if !matches!(self.payload, None | Some(Payload::Indexation(_))) {
            // Unwrap is fine because we just checked that the Option is not None.
            return Err(Error::InvalidPayloadKind(self.payload.unwrap().kind()));
        }

        for input in self.inputs.iter() {
            match input {
                Input::Utxo(u) => {
                    if self.inputs.iter().filter(|i| *i == input).count() > 1 {
                        return Err(Error::DuplicateUtxo(u.clone()));
                    }
                }
                _ => return Err(Error::InvalidInputKind(input.kind())),
            }
        }

        let mut total_amount: u64 = 0;

        for output in self.outputs.iter() {
            let amount = match output {
                Output::Simple(output) => output.amount(),
                Output::Extended(output) => output.amount(),
                Output::Alias(output) => output.amount(),
                Output::Foundry(output) => output.amount(),
                Output::Nft(output) => output.amount(),
                _ => return Err(Error::InvalidOutputKind(output.kind())),
            };

            total_amount = total_amount
                .checked_add(amount)
                .ok_or_else(|| Error::InvalidAccumulatedOutput((total_amount + amount) as u128))?;

            // Accumulated output balance must not exceed the total supply of tokens.
            if total_amount > IOTA_SUPPLY {
                return Err(Error::InvalidAccumulatedOutput(total_amount as u128));
            }
        }

        Ok(RegularTransactionEssence {
            inputs: self.inputs.into_boxed_slice(),
            outputs: self.outputs.into_boxed_slice(),
            payload: self.payload,
        })
    }
}

/// A transaction regular essence consuming inputs, creating outputs and carrying an optional payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct RegularTransactionEssence {
    inputs: Box<[Input]>,
    outputs: Box<[Output]>,
    payload: Option<Payload>,
}

impl RegularTransactionEssence {
    /// The essence kind of a [`RegularTransactionEssence`].
    pub const KIND: u8 = 0;

    /// Create a new [`RegularTransactionEssenceBuilder`] to build a [`RegularTransactionEssence`].
    pub fn builder() -> RegularTransactionEssenceBuilder {
        RegularTransactionEssenceBuilder::new()
    }

    /// Return the inputs of a [`RegularTransactionEssence`].
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// Return the outputs of a [`RegularTransactionEssence`].
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    /// Return the optional payload of a [`RegularTransactionEssence`].
    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }
}

impl Packable for RegularTransactionEssence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len()
            + self.inputs.iter().map(Packable::packed_len).sum::<usize>()
            + 0u16.packed_len()
            + self.outputs.iter().map(Packable::packed_len).sum::<usize>()
            + option_payload_packed_len(self.payload.as_ref())
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.inputs.len() as u16).pack(writer)?;
        for input in self.inputs.iter() {
            input.pack(writer)?;
        }
        (self.outputs.len() as u16).pack(writer)?;
        for output in self.outputs.iter() {
            output.pack(writer)?;
        }
        option_payload_pack(writer, self.payload.as_ref())?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let inputs_len = u16::unpack_inner::<R, CHECK>(reader)?;

        if CHECK && !INPUT_COUNT_RANGE.contains(&inputs_len) {
            return Err(Error::InvalidInputOutputCount(inputs_len));
        }

        let mut inputs = Vec::with_capacity(inputs_len as usize);
        for _ in 0..inputs_len {
            inputs.push(Input::unpack_inner::<R, CHECK>(reader)?);
        }

        let outputs_len = u16::unpack_inner::<R, CHECK>(reader)?;

        if CHECK && !OUTPUT_COUNT_RANGE.contains(&outputs_len) {
            return Err(Error::InvalidInputOutputCount(outputs_len));
        }

        let mut outputs = Vec::with_capacity(outputs_len as usize);
        for _ in 0..outputs_len {
            outputs.push(Output::unpack_inner::<R, CHECK>(reader)?);
        }

        let mut builder = Self::builder().with_inputs(inputs).with_outputs(outputs);

        if let (_, Some(payload)) = option_payload_unpack::<R, CHECK>(reader)? {
            builder = builder.with_payload(payload);
        }

        builder.finish()
    }
}

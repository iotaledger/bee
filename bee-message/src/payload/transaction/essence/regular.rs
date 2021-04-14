// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants::{INPUT_OUTPUT_COUNT_RANGE, IOTA_SUPPLY},
    input::Input,
    output::Output,
    payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, Payload},
    Error,
};

use bee_common::{
    ord::is_sorted,
    packable::{Packable, Read, Write},
};

use alloc::{boxed::Box, vec::Vec};

/// A transaction regular essence consuming inputs, creating outputs and carrying an optional payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RegularEssence {
    inputs: Box<[Input]>,
    outputs: Box<[Output]>,
    payload: Option<Payload>,
}

impl RegularEssence {
    /// The essence kind of a `RegularEssence`
    pub const KIND: u8 = 0;

    /// Create a new `RegularEssenceBuilder` to build a `RegularEssence`.
    pub fn builder() -> RegularEssenceBuilder {
        RegularEssenceBuilder::new()
    }

    /// Return the inputs of a `RegularEssence`.
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// Return the outputs of a `RegularEssence`.
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    /// Return the optional payload of a `RegularEssence`.
    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }
}

impl Packable for RegularEssence {
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
        let inputs_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK && !INPUT_OUTPUT_COUNT_RANGE.contains(&inputs_len) {
            return Err(Error::InvalidInputOutputCount(inputs_len));
        }

        let mut inputs = Vec::with_capacity(inputs_len);
        for _ in 0..inputs_len {
            inputs.push(Input::unpack_inner::<R, CHECK>(reader)?);
        }

        let outputs_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK && !INPUT_OUTPUT_COUNT_RANGE.contains(&outputs_len) {
            return Err(Error::InvalidInputOutputCount(outputs_len));
        }

        let mut outputs = Vec::with_capacity(outputs_len);
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

/// A builder to build a `RegularEssence`.
#[derive(Debug, Default)]
pub struct RegularEssenceBuilder {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    payload: Option<Payload>,
}

impl RegularEssenceBuilder {
    /// Creates a new `RegularEssenceBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds inputs to a `RegularEssenceBuilder`
    pub fn with_inputs(mut self, inputs: Vec<Input>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Add an input to a `RegularEssenceBuilder`.
    pub fn add_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    /// Add outputs to a `RegularEssenceBuilder`.
    pub fn with_outputs(mut self, outputs: Vec<Output>) -> Self {
        self.outputs = outputs;
        self
    }

    /// Add an output to a `RegularEssenceBuilder`.
    pub fn add_output(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }

    /// Add a payload to a `RegularEssenceBuilder`.
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Finishes a `RegularEssenceBuilder` into a `RegularEssence`.
    pub fn finish(self) -> Result<RegularEssence, Error> {
        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.inputs.len()) {
            return Err(Error::InvalidInputOutputCount(self.inputs.len()));
        }

        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.outputs.len()) {
            return Err(Error::InvalidInputOutputCount(self.outputs.len()));
        }

        if !matches!(self.payload, None | Some(Payload::Indexation(_))) {
            // Unwrap is fine because we just checked that the Option is not None.
            return Err(Error::InvalidPayloadKind(self.payload.unwrap().kind()));
        }

        // Inputs validation

        for input in self.inputs.iter() {
            match input {
                Input::Utxo(_) => {
                    if self.inputs.iter().filter(|i| *i == input).count() > 1 {
                        return Err(Error::DuplicateError);
                    }
                }
                _ => return Err(Error::InvalidInputKind(input.kind())),
            }
        }

        // Inputs must be lexicographically sorted in their serialised forms.
        if !is_sorted(self.inputs.iter().map(Packable::pack_new)) {
            return Err(Error::TransactionInputsNotSorted);
        }

        // Outputs validation

        let mut total: u64 = 0;

        // TODO iteration-based or memory-based ?

        for output in self.outputs.iter() {
            match output {
                Output::SignatureLockedSingle(single) => {
                    // The addresses must be unique in the set of SignatureLockedSingleOutputs.
                    if self
                        .outputs
                        .iter()
                        .filter(|o| matches!(o, Output::SignatureLockedSingle(s) if s.address() == single.address()))
                        .count()
                        > 1
                    {
                        return Err(Error::DuplicateError);
                    }

                    total = total
                        .checked_add(single.amount())
                        .ok_or_else(|| Error::InvalidAccumulatedOutput((total + single.amount()) as u128))?;
                }
                Output::SignatureLockedDustAllowance(dust_allowance) => {
                    // The addresses must be unique in the set of SignatureLockedDustAllowanceOutputs.
                    if self
                        .outputs
                        .iter()
                        .filter(
                            |o| matches!(o, Output::SignatureLockedDustAllowance(s) if s.address() == dust_allowance.address()),
                        )
                        .count()
                        > 1
                    {
                        return Err(Error::DuplicateError);
                    }

                    total = total
                        .checked_add(dust_allowance.amount())
                        .ok_or_else(|| Error::InvalidAccumulatedOutput((total + dust_allowance.amount()) as u128))?;
                }
                _ => return Err(Error::InvalidOutputKind(output.kind())),
            }

            // Accumulated output balance must not exceed the total supply of tokens.
            if total > IOTA_SUPPLY {
                return Err(Error::InvalidAccumulatedOutput(total as u128));
            }
        }

        // Outputs must be lexicographically sorted in their serialised forms.
        if !is_sorted(self.outputs.iter().map(Packable::pack_new)) {
            return Err(Error::TransactionOutputsNotSorted);
        }

        Ok(RegularEssence {
            inputs: self.inputs.into_boxed_slice(),
            outputs: self.outputs.into_boxed_slice(),
            payload: self.payload,
        })
    }
}

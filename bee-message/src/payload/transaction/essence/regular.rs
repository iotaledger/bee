// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constant::IOTA_SUPPLY,
    input::{Input, INPUT_COUNT_RANGE},
    output::{Output, OUTPUT_COUNT_RANGE},
    payload::{OptionalPayload, Payload},
    Error,
};

use packable::{bounded::BoundedU16, prefix::BoxedSlicePrefix, Packable};

use alloc::vec::Vec;

/// A builder to build a [`RegularTransactionEssence`].
#[derive(Debug, Default)]
#[must_use]
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
        let inputs: BoxedSlicePrefix<Input, InputCount> = self
            .inputs
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidInputCount)?;
        let outputs: BoxedSlicePrefix<Output, OutputCount> = self
            .outputs
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidOutputCount)?;
        let payload = OptionalPayload::from(self.payload);

        verify_inputs::<true>(&inputs)?;
        verify_outputs::<true>(&outputs)?;
        verify_payload::<true>(&payload)?;

        Ok(RegularTransactionEssence {
            inputs,
            outputs,
            payload,
        })
    }
}

pub(crate) type InputCount = BoundedU16<{ *INPUT_COUNT_RANGE.start() }, { *INPUT_COUNT_RANGE.end() }>;
pub(crate) type OutputCount = BoundedU16<{ *OUTPUT_COUNT_RANGE.start() }, { *OUTPUT_COUNT_RANGE.end() }>;

/// A transaction regular essence consuming inputs, creating outputs and carrying an optional payload.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct RegularTransactionEssence {
    #[packable(verify_with = verify_inputs)]
    #[packable(unpack_error_with = |e| e.unwrap_packable_or_else(|p| Error::InvalidInputCount(p.into())))]
    inputs: BoxedSlicePrefix<Input, InputCount>,
    #[packable(verify_with = verify_outputs)]
    #[packable(unpack_error_with = |e| e.unwrap_packable_or_else(|p| Error::InvalidOutputCount(p.into())))]
    outputs: BoxedSlicePrefix<Output, OutputCount>,
    #[packable(verify_with = verify_payload)]
    payload: OptionalPayload,
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

fn verify_inputs<const VERIFY: bool>(inputs: &[Input]) -> Result<(), Error> {
    for input in inputs.iter() {
        match input {
            Input::Utxo(u) => {
                if inputs.iter().filter(|i| *i == input).count() > 1 {
                    return Err(Error::DuplicateUtxo(u.clone()));
                }
            }
            _ => return Err(Error::InvalidInputKind(input.kind())),
        }
    }

    Ok(())
}

fn verify_outputs<const VERIFY: bool>(outputs: &[Output]) -> Result<(), Error> {
    let mut total_amount: u64 = 0;

    for output in outputs.iter() {
        let amount = match output {
            Output::Basic(output) => output.amount(),
            Output::Alias(output) => output.amount(),
            Output::Foundry(output) => output.amount(),
            Output::Nft(output) => output.amount(),
            _ => return Err(Error::InvalidOutputKind(output.kind())),
        };

        total_amount = total_amount
            .checked_add(amount)
            .ok_or(Error::InvalidAccumulatedOutput((total_amount + amount) as u128))?;

        // Accumulated output balance must not exceed the total supply of tokens.
        if total_amount > IOTA_SUPPLY {
            return Err(Error::InvalidAccumulatedOutput(total_amount as u128));
        }
    }

    Ok(())
}

fn verify_payload<const VERIFY: bool>(payload: &OptionalPayload) -> Result<(), Error> {
    match &payload.0 {
        Some(Payload::TaggedData(_)) | None => Ok(()),
        Some(payload) => Err(Error::InvalidPayloadKind(payload.kind())),
    }
}

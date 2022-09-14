// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;

use hashbrown::HashSet;
use packable::{bounded::BoundedU16, prefix::BoxedSlicePrefix, Packable};

use crate::{
    input::{Input, INPUT_COUNT_RANGE},
    output::{InputsCommitment, NativeTokens, Output, OUTPUT_COUNT_RANGE},
    payload::{OptionalPayload, Payload},
    protocol::ProtocolParameters,
    Error,
};

/// A builder to build a [`RegularTransactionEssence`].
#[derive(Debug, Clone)]
#[must_use]
pub struct RegularTransactionEssenceBuilder {
    inputs: Vec<Input>,
    inputs_commitment: InputsCommitment,
    outputs: Vec<Output>,
    payload: Option<Payload>,
}

impl RegularTransactionEssenceBuilder {
    /// Creates a new [`RegularTransactionEssenceBuilder`].
    pub fn new(inputs_commitment: InputsCommitment) -> Self {
        Self {
            inputs: Vec::new(),
            inputs_commitment,
            outputs: Vec::new(),
            payload: None,
        }
    }

    /// Adds inputs to a [`RegularTransactionEssenceBuilder`].
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
    pub fn finish(self, protocol_parameters: &ProtocolParameters) -> Result<RegularTransactionEssence, Error> {
        let inputs: BoxedSlicePrefix<Input, InputCount> = self
            .inputs
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidInputCount)?;

        verify_inputs::<true>(&inputs)?;

        let outputs: BoxedSlicePrefix<Output, OutputCount> = self
            .outputs
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidOutputCount)?;

        verify_outputs::<true>(&outputs, protocol_parameters)?;

        let payload = OptionalPayload::from(self.payload);

        verify_payload::<true>(&payload)?;

        Ok(RegularTransactionEssence {
            network_id: protocol_parameters.network_id(),
            inputs,
            inputs_commitment: self.inputs_commitment,
            outputs,
            payload,
        })
    }
}

pub(crate) type InputCount = BoundedU16<{ *INPUT_COUNT_RANGE.start() }, { *INPUT_COUNT_RANGE.end() }>;
pub(crate) type OutputCount = BoundedU16<{ *OUTPUT_COUNT_RANGE.start() }, { *OUTPUT_COUNT_RANGE.end() }>;

/// A transaction regular essence consuming inputs, creating outputs and carrying an optional payload.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
#[packable(unpack_visitor = ProtocolParameters)]
pub struct RegularTransactionEssence {
    /// The unique value denoting whether the block was meant for mainnet, testnet, or a private network.
    #[packable(verify_with = verify_network_id)]
    network_id: u64,
    #[packable(verify_with = verify_inputs_packable)]
    #[packable(unpack_error_with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidInputCount(p.into())))]
    inputs: BoxedSlicePrefix<Input, InputCount>,
    /// BLAKE2b-256 hash of the serialized outputs referenced in inputs by their OutputId.
    inputs_commitment: InputsCommitment,
    #[packable(verify_with = verify_outputs)]
    #[packable(unpack_error_with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidOutputCount(p.into())))]
    outputs: BoxedSlicePrefix<Output, OutputCount>,
    #[packable(verify_with = verify_payload_packable)]
    payload: OptionalPayload,
}

impl RegularTransactionEssence {
    /// The essence kind of a [`RegularTransactionEssence`].
    pub const KIND: u8 = 1;

    /// Creates a new [`RegularTransactionEssenceBuilder`] to build a [`RegularTransactionEssence`].
    pub fn builder(inputs_commitment: InputsCommitment) -> RegularTransactionEssenceBuilder {
        RegularTransactionEssenceBuilder::new(inputs_commitment)
    }

    /// Returns the network ID of a [`RegularTransactionEssence`].
    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    /// Returns the inputs of a [`RegularTransactionEssence`].
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// Returns the inputs commitment of a [`RegularTransactionEssence`].
    pub fn inputs_commitment(&self) -> &InputsCommitment {
        &self.inputs_commitment
    }

    /// Returns the outputs of a [`RegularTransactionEssence`].
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    /// Returns the optional payload of a [`RegularTransactionEssence`].
    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }
}

fn verify_network_id<const VERIFY: bool>(network_id: &u64, visitor: &ProtocolParameters) -> Result<(), Error> {
    if VERIFY {
        let expected = visitor.network_id();

        if *network_id != expected {
            return Err(Error::NetworkIdMismatch {
                expected,
                actual: *network_id,
            });
        }
    }

    Ok(())
}

fn verify_inputs<const VERIFY: bool>(inputs: &[Input]) -> Result<(), Error> {
    if VERIFY {
        let mut seen_utxos = HashSet::new();

        for input in inputs.iter() {
            match input {
                Input::Utxo(utxo) => {
                    if !seen_utxos.insert(utxo) {
                        return Err(Error::DuplicateUtxo(utxo.clone()));
                    }
                }
                _ => return Err(Error::InvalidInputKind(input.kind())),
            }
        }
    }

    Ok(())
}

fn verify_inputs_packable<const VERIFY: bool>(inputs: &[Input], _visitor: &ProtocolParameters) -> Result<(), Error> {
    verify_inputs::<VERIFY>(inputs)
}

fn verify_outputs<const VERIFY: bool>(outputs: &[Output], visitor: &ProtocolParameters) -> Result<(), Error> {
    if VERIFY {
        let mut amount_sum: u64 = 0;
        let mut native_tokens_count: u8 = 0;

        for output in outputs.iter() {
            let (amount, native_tokens) = match output {
                Output::Basic(output) => (output.amount(), output.native_tokens()),
                Output::Alias(output) => (output.amount(), output.native_tokens()),
                Output::Foundry(output) => (output.amount(), output.native_tokens()),
                Output::Nft(output) => (output.amount(), output.native_tokens()),
                _ => return Err(Error::InvalidOutputKind(output.kind())),
            };

            amount_sum = amount_sum
                .checked_add(amount)
                .ok_or(Error::InvalidTransactionAmountSum(amount_sum as u128 + amount as u128))?;

            // Accumulated output balance must not exceed the total supply of tokens.
            if amount_sum > visitor.token_supply() {
                return Err(Error::InvalidTransactionAmountSum(amount_sum as u128));
            }

            native_tokens_count = native_tokens_count.checked_add(native_tokens.len() as u8).ok_or(
                Error::InvalidTransactionNativeTokensCount(native_tokens_count as u16 + native_tokens.len() as u16),
            )?;

            if native_tokens_count > NativeTokens::COUNT_MAX {
                return Err(Error::InvalidTransactionNativeTokensCount(native_tokens_count as u16));
            }

            output.verify_storage_deposit(visitor.rent_structure().clone(), visitor.token_supply())?;
        }
    }

    Ok(())
}

fn verify_payload<const VERIFY: bool>(payload: &OptionalPayload) -> Result<(), Error> {
    if VERIFY {
        match &payload.0 {
            Some(Payload::TaggedData(_)) | None => Ok(()),
            Some(payload) => Err(Error::InvalidPayloadKind(payload.kind())),
        }
    } else {
        Ok(())
    }
}

fn verify_payload_packable<const VERIFY: bool>(
    payload: &OptionalPayload,
    _visitor: &ProtocolParameters,
) -> Result<(), Error> {
    verify_payload::<VERIFY>(payload)
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use core::str::FromStr;

    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{
        error::dto::DtoError,
        input::dto::InputDto,
        output::dto::{try_from_output_dto_for_output, OutputDto},
        payload::dto::PayloadDto,
    };

    /// Describes the essence data making up a transaction by defining its inputs and outputs and an optional payload.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct RegularTransactionEssenceDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "networkId")]
        pub network_id: String,
        pub inputs: Vec<InputDto>,
        #[serde(rename = "inputsCommitment")]
        pub inputs_commitment: String,
        pub outputs: Vec<OutputDto>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub payload: Option<PayloadDto>,
    }

    impl From<&RegularTransactionEssence> for RegularTransactionEssenceDto {
        fn from(value: &RegularTransactionEssence) -> Self {
            RegularTransactionEssenceDto {
                kind: RegularTransactionEssence::KIND,
                network_id: value.network_id().to_string(),
                inputs: value.inputs().iter().map(Into::into).collect::<Vec<_>>(),
                inputs_commitment: value.inputs_commitment().to_string(),
                outputs: value.outputs().iter().map(Into::into).collect::<Vec<_>>(),
                payload: match value.payload() {
                    Some(Payload::TaggedData(i)) => Some(PayloadDto::TaggedData(Box::new(i.as_ref().into()))),
                    Some(_) => unimplemented!(),
                    None => None,
                },
            }
        }
    }

    pub fn try_from_regular_transaction_essence_dto_for_regular_transaction_essence(
        value: &RegularTransactionEssenceDto,
        protocol_parameters: &ProtocolParameters,
    ) -> Result<RegularTransactionEssence, DtoError> {
        let inputs = value
            .inputs
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Input>, DtoError>>()?;
        let outputs = value
            .outputs
            .iter()
            .map(|o| try_from_output_dto_for_output(o, protocol_parameters.token_supply()))
            .collect::<Result<Vec<Output>, DtoError>>()?;

        let mut builder = RegularTransactionEssence::builder(InputsCommitment::from_str(&value.inputs_commitment)?)
            .with_inputs(inputs)
            .with_outputs(outputs);
        builder = if let Some(p) = &value.payload {
            if let PayloadDto::TaggedData(i) = p {
                builder.with_payload(Payload::TaggedData(Box::new((i.as_ref()).try_into()?)))
            } else {
                return Err(DtoError::InvalidField("payload"));
            }
        } else {
            builder
        };

        builder.finish(protocol_parameters).map_err(Into::into)
    }
}

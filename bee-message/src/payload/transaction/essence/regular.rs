// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constant::IOTA_SUPPLY,
    input::{Input, INPUT_COUNT_RANGE},
    output::{Output, OUTPUT_COUNT_RANGE},
    payload::{OptionalPayload, Payload},
    Error,
};

use bee_packable::{
    bounded::BoundedU16,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::BoxedSlicePrefix,
    unpacker::Unpacker,
    Packable,
};

use alloc::vec::Vec;

pub(crate) type InputCount = BoundedU16<{ *INPUT_COUNT_RANGE.start() }, { *INPUT_COUNT_RANGE.end() }>;
pub(crate) type OutputCount = BoundedU16<{ *OUTPUT_COUNT_RANGE.start() }, { *OUTPUT_COUNT_RANGE.end() }>;

/// A transaction regular essence consuming inputs, creating outputs and carrying an optional payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct RegularTransactionEssence {
    inputs: BoxedSlicePrefix<Input, InputCount>,
    outputs: BoxedSlicePrefix<Output, OutputCount>,
    payload: OptionalPayload,
}

impl RegularTransactionEssence {
    /// The essence kind of a [`RegularTransactionEssence`].
    pub const KIND: u8 = 0;

    /// Create a new [`RegularTransactionEssence`].
    pub fn new(inputs: Vec<Input>, outputs: Vec<Output>, payload: Option<Payload>) -> Result<Self, Error> {
        let inputs = inputs.into_boxed_slice().try_into().map_err(Error::InvalidInputCount)?;
        let outputs = outputs
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidOutputCount)?;

        Self::from_boxed_slice_prefixes(inputs, outputs, payload)
    }

    fn from_boxed_slice_prefixes(
        inputs: BoxedSlicePrefix<Input, InputCount>,
        outputs: BoxedSlicePrefix<Output, OutputCount>,
        payload: Option<Payload>,
    ) -> Result<Self, Error> {
        if !matches!(payload, None | Some(Payload::Indexation(_))) {
            // Unwrap is fine because we just checked that the Option is not None.
            return Err(Error::InvalidPayloadKind(payload.unwrap().kind()));
        }

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

        let mut total_amount: u64 = 0;

        for output in outputs.iter() {
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
            inputs,
            outputs,
            payload: payload.into(),
        })
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
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.inputs.pack(packer)?;
        self.outputs.pack(packer)?;
        self.payload.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let inputs = BoxedSlicePrefix::<Input, InputCount>::unpack::<_, VERIFY>(unpacker).map_packable_err(|err| {
            err.unwrap_packable_or_else(|prefix_err| Error::InvalidInputCount(prefix_err.into()))
        })?;
        let outputs =
            BoxedSlicePrefix::<Output, OutputCount>::unpack::<_, VERIFY>(unpacker).map_packable_err(|err| {
                err.unwrap_packable_or_else(|prefix_err| Error::InvalidOutputCount(prefix_err.into()))
            })?;
        let payload = OptionalPayload::unpack::<_, VERIFY>(unpacker)?.into();

        Self::from_boxed_slice_prefixes(inputs, outputs, payload).map_err(UnpackError::Packable)
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::{
        transaction::{
            constants::{INPUT_OUTPUT_COUNT_RANGE, INPUT_OUTPUT_INDEX_RANGE},
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

        let payload_len = u32::unpack(reader)? as usize;
        let payload = if payload_len > 0 {
            let payload = Payload::unpack(reader)?;
            if payload_len != payload.packed_len() {
                return Err(Self::Error::InvalidAnnouncedLength(payload_len, payload.packed_len()));
            }

            Some(payload)
        } else {
            None
        };

        // TODO check payload type

        Ok(Self {
            inputs: inputs.into_boxed_slice(),
            outputs: outputs.into_boxed_slice(),
            payload,
        })
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

    pub fn add_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
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
        // Inputs validation

        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.inputs.len()) {
            return Err(Error::CountError);
        }

        for i in self.inputs.iter() {
            // Input Type value must be 0, denoting an UTXO Input.
            match i {
                Input::UTXO(u) => {
                    // Transaction Output Index must be 0 ≤ x < 127
                    if !INPUT_OUTPUT_INDEX_RANGE.contains(&u.output_id().index()) {
                        return Err(Error::CountError);
                    }

                    // Every combination of Transaction ID + Transaction Output Index must be unique in the inputs set.
                    if self.inputs.iter().filter(|j| *j == i).count() > 1 {
                        return Err(Error::DuplicateError);
                    }
                }
            }
        }

        // Output validation

        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.outputs.len()) {
            return Err(Error::CountError);
        }

        let mut total = 0;
        for i in self.outputs.iter() {
            // Output Type must be 0, denoting a SignatureLockedSingle.
            match i {
                Output::SignatureLockedSingle(u) => {
                    // Address Type must either be 0 or 1, denoting a WOTS- or Ed25519 address.

                    // If Address is of type WOTS address, its bytes must be valid T5B1 bytes.

                    // The Address must be unique in the set of SigLockedSingleDeposits
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

                    // Amount must be > 0
                    let amount = u.amount().get();
                    if amount == 0 {
                        return Err(Error::AmountError);
                    }

                    total += amount;
                }
            }
        }

        // Accumulated output balance must not exceed the total supply of tokens 2'779'530'283'277'761
        if total > 2779530283277761 {
            return Err(Error::AmountError);
        }

        // Payload Length must be 0 (to indicate that there's no payload) or be valid for the specified payload type.
        // Payload Type must be one of the supported payload types if Payload Length is not 0.
        // TODO check payload type

        self.inputs.sort();
        self.outputs.sort();

        Ok(TransactionEssence {
            inputs: self.inputs.into_boxed_slice(),
            outputs: self.outputs.into_boxed_slice(),
            payload: self.payload,
        })
    }
}

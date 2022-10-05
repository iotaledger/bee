// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the treasury payload.

use crate::{
    input::{Input, TreasuryInput},
    output::{Output, TreasuryOutput},
    protocol::ProtocolParameters,
    Error,
};

/// [`TreasuryTransactionPayload`] represents a transaction which moves funds from the treasury.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_visitor = ProtocolParameters)]
pub struct TreasuryTransactionPayload {
    #[packable(verify_with = verify_input)]
    input: Input,
    #[packable(verify_with = verify_output)]
    output: Output,
}

impl TreasuryTransactionPayload {
    /// The payload kind of a [`TreasuryTransactionPayload`].
    pub const KIND: u32 = 4;

    /// Creates a new [`TreasuryTransactionPayload`].
    pub fn new(input: TreasuryInput, output: TreasuryOutput) -> Result<Self, Error> {
        Ok(Self {
            input: input.into(),
            output: output.into(),
        })
    }

    /// Returns the input of a [`TreasuryTransactionPayload`].
    pub fn input(&self) -> &TreasuryInput {
        if let Input::Treasury(ref input) = self.input {
            input
        } else {
            // It has already been validated at construction that `input` is a `TreasuryInput`.
            unreachable!()
        }
    }

    /// Returns the output of a [`TreasuryTransactionPayload`].
    pub fn output(&self) -> &TreasuryOutput {
        if let Output::Treasury(ref output) = self.output {
            output
        } else {
            // It has already been validated at construction that `output` is a `TreasuryOutput`.
            unreachable!()
        }
    }
}

fn verify_input<const VERIFY: bool>(input: &Input, _: &ProtocolParameters) -> Result<(), Error> {
    if VERIFY && !matches!(input, Input::Treasury(_)) {
        Err(Error::InvalidInputKind(input.kind()))
    } else {
        Ok(())
    }
}

fn verify_output<const VERIFY: bool>(output: &Output, _: &ProtocolParameters) -> Result<(), Error> {
    if VERIFY && !matches!(output, Output::Treasury(_)) {
        Err(Error::InvalidOutputKind(output.kind()))
    } else {
        Ok(())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{
        error::dto::DtoError,
        input::dto::{InputDto, TreasuryInputDto},
        output::dto::{OutputDto, TreasuryOutputDto},
    };

    /// The payload type to define a treasury transaction.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct TreasuryTransactionPayloadDto {
        #[serde(rename = "type")]
        pub kind: u32,
        pub input: InputDto,
        pub output: OutputDto,
    }

    impl From<&TreasuryTransactionPayload> for TreasuryTransactionPayloadDto {
        fn from(value: &TreasuryTransactionPayload) -> Self {
            TreasuryTransactionPayloadDto {
                kind: TreasuryTransactionPayload::KIND,
                input: InputDto::Treasury(TreasuryInputDto::from(value.input())),
                output: OutputDto::Treasury(TreasuryOutputDto::from(value.output())),
            }
        }
    }

    impl TreasuryTransactionPayload {
        pub fn try_from_dto(
            value: &TreasuryTransactionPayloadDto,
            token_supply: u64,
        ) -> Result<TreasuryTransactionPayload, DtoError> {
            Ok(TreasuryTransactionPayload::new(
                if let InputDto::Treasury(ref input) = value.input {
                    input.try_into()?
                } else {
                    return Err(DtoError::InvalidField("input"));
                },
                if let OutputDto::Treasury(ref output) = value.output {
                    TreasuryOutput::try_from_dto(output, token_supply)?
                } else {
                    return Err(DtoError::InvalidField("output"));
                },
            )?)
        }

        pub fn try_from_dto_unverified(
            value: &TreasuryTransactionPayloadDto,
        ) -> Result<TreasuryTransactionPayload, DtoError> {
            Ok(TreasuryTransactionPayload::new(
                if let InputDto::Treasury(ref input) = value.input {
                    input.try_into()?
                } else {
                    return Err(DtoError::InvalidField("input"));
                },
                if let OutputDto::Treasury(ref output) = value.output {
                    TreasuryOutput::try_from_dto_unverified(output)?
                } else {
                    return Err(DtoError::InvalidField("output"));
                },
            )?)
        }
    }
}

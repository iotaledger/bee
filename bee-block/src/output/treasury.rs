// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{protocol::ProtocolParameters, Error};

/// [`TreasuryOutput`] is an output which holds the treasury of a network.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
#[packable(unpack_visitor = ProtocolParameters)]
pub struct TreasuryOutput {
    #[packable(verify_with = verify_amount_packable)]
    amount: u64,
}

impl TreasuryOutput {
    /// The [`Output`](crate::output::Output) kind of a [`TreasuryOutput`].
    pub const KIND: u8 = 2;

    /// Creates a new [`TreasuryOutput`].
    pub fn new(amount: u64, token_supply: u64) -> Result<Self, Error> {
        verify_amount::<true>(&amount, &token_supply)?;

        Ok(Self { amount })
    }

    /// Returns the amount of a [`TreasuryOutput`].
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

fn verify_amount<const VERIFY: bool>(amount: &u64, token_supply: &u64) -> Result<(), Error> {
    if VERIFY && amount > token_supply {
        Err(Error::InvalidTreasuryOutputAmount(*amount))
    } else {
        Ok(())
    }
}

fn verify_amount_packable<const VERIFY: bool>(
    amount: &u64,
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    verify_amount::<VERIFY>(amount, &protocol_parameters.token_supply())
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    /// Describes a treasury output.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct TreasuryOutputDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub amount: String,
    }

    impl From<&TreasuryOutput> for TreasuryOutputDto {
        fn from(value: &TreasuryOutput) -> Self {
            Self {
                kind: TreasuryOutput::KIND,
                amount: value.amount().to_string(),
            }
        }
    }

    impl TreasuryOutput {
        pub fn try_from_dto(value: &TreasuryOutputDto, token_supply: u64) -> Result<TreasuryOutput, DtoError> {
            Ok(TreasuryOutput::new(
                value
                    .amount
                    .parse::<u64>()
                    .map_err(|_| DtoError::InvalidField("amount"))?,
                token_supply,
            )?)
        }

        pub fn try_from_dto_unverified(value: &TreasuryOutputDto) -> Result<TreasuryOutput, DtoError> {
            Ok(TreasuryOutput {
                amount: value
                    .amount
                    .parse::<u64>()
                    .map_err(|_| DtoError::InvalidField("amount"))?,
            })
        }
    }
}

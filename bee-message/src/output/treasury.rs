// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::RangeInclusive;

use packable::bounded::BoundedU64;

use crate::{constant::TOKEN_SUPPLY, Error};

pub(crate) type TreasuryOutputAmount =
    BoundedU64<{ *TreasuryOutput::AMOUNT_RANGE.start() }, { *TreasuryOutput::AMOUNT_RANGE.end() }>;

/// [`TreasuryOutput`] is an output which holds the treasury of a network.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = Error::InvalidTreasuryOutputAmount)]
pub struct TreasuryOutput {
    amount: TreasuryOutputAmount,
}

impl TreasuryOutput {
    /// The [`Output`](crate::output::Output) kind of a [`TreasuryOutput`].
    pub const KIND: u8 = 2;
    /// The allowed range of the amount of a [`TreasuryOutput`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = 0..=TOKEN_SUPPLY;

    /// Creates a new [`TreasuryOutput`].
    pub fn new(amount: u64) -> Result<Self, Error> {
        amount
            .try_into()
            .map(|amount| Self { amount })
            .map_err(Error::InvalidTreasuryOutputAmount)
    }

    /// Returns the amount of a [`TreasuryOutput`].
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }
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

    impl TryFrom<&TreasuryOutputDto> for TreasuryOutput {
        type Error = DtoError;

        fn try_from(value: &TreasuryOutputDto) -> Result<Self, Self::Error> {
            Ok(Self::new(
                value
                    .amount
                    .parse::<u64>()
                    .map_err(|_| DtoError::InvalidField("amount"))?,
            )?)
        }
    }
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod treasury;
mod utxo;

use core::ops::RangeInclusive;

use derive_more::From;

pub use self::{treasury::TreasuryInput, utxo::UtxoInput};
use crate::Error;

/// The maximum number of inputs of a transaction.
pub const INPUT_COUNT_MAX: u16 = 128;
/// The range of valid numbers of inputs of a transaction.
pub const INPUT_COUNT_RANGE: RangeInclusive<u16> = 1..=INPUT_COUNT_MAX; // [1..128]
/// The maximum index of inputs of a transaction.
pub const INPUT_INDEX_MAX: u16 = INPUT_COUNT_MAX - 1; // 127
/// The range of valid indices of inputs of a transaction.
pub const INPUT_INDEX_RANGE: RangeInclusive<u16> = 0..=INPUT_INDEX_MAX; // [0..127]

/// A generic input supporting different input kinds.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidInputKind)]
pub enum Input {
    /// A UTXO input.
    #[packable(tag = UtxoInput::KIND)]
    Utxo(UtxoInput),
    /// A treasury input.
    #[packable(tag = TreasuryInput::KIND)]
    Treasury(TreasuryInput),
}

impl Input {
    /// Returns the input kind of an `Input`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Utxo(_) => UtxoInput::KIND,
            Self::Treasury(_) => TreasuryInput::KIND,
        }
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    pub use super::{treasury::dto::TreasuryInputDto, utxo::dto::UtxoInputDto};
    use crate::error::dto::DtoError;

    /// Describes all the different input types.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum InputDto {
        Utxo(UtxoInputDto),
        Treasury(TreasuryInputDto),
    }

    impl From<&Input> for InputDto {
        fn from(value: &Input) -> Self {
            match value {
                Input::Utxo(u) => InputDto::Utxo(u.into()),
                Input::Treasury(t) => InputDto::Treasury(t.into()),
            }
        }
    }

    impl TryFrom<&InputDto> for Input {
        type Error = DtoError;

        fn try_from(value: &InputDto) -> Result<Self, Self::Error> {
            match value {
                InputDto::Utxo(u) => Ok(Input::Utxo(u.try_into()?)),
                InputDto::Treasury(t) => Ok(Input::Treasury(t.try_into()?)),
            }
        }
    }
}

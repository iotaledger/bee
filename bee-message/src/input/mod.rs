// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod treasury;
mod utxo;

pub use treasury::TreasuryInput;
pub use utxo::UtxoInput;

use crate::Error;

use bee_packable::Packable;

/// A generic input supporting different input kinds.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = Error::InvalidInputKind)]
#[packable(unpack_error = Error)]
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

impl From<UtxoInput> for Input {
    fn from(input: UtxoInput) -> Self {
        Self::Utxo(input)
    }
}

impl From<TreasuryInput> for Input {
    fn from(input: TreasuryInput) -> Self {
        Self::Treasury(input)
    }
}

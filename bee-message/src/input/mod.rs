// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod treasury;
mod utxo;

pub use treasury::TreasuryInput;
pub use utxo::UtxoInput;

use crate::Error;

use derive_more::From;

use core::ops::RangeInclusive;

/// The maximum number of inputs of a transaction.
pub const INPUT_COUNT_MAX: u16 = 127;
/// The range of valid numbers of inputs of a transaction.
pub const INPUT_COUNT_RANGE: RangeInclusive<u16> = 1..=INPUT_COUNT_MAX; // [1..127]
/// The maximum index of inputs of a transaction.
pub const INPUT_INDEX_MAX: u16 = INPUT_COUNT_MAX - 1; // 126
/// The range of valid indices of inputs of a transaction.
pub const INPUT_INDEX_RANGE: RangeInclusive<u16> = 0..=INPUT_INDEX_MAX; // [0..126]

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

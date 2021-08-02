// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of inputs.

mod utxo;

pub use utxo::UtxoInput;

use crate::error::{MessageUnpackError, ValidationError};

use bee_packable::Packable;

use core::{
    convert::Infallible,
    fmt,
    ops::{Range, RangeInclusive},
};

/// The maximum number of inputs of a transaction.
pub const INPUT_COUNT_MAX: usize = 127;
/// The range of valid numbers of inputs of a transaction.
pub const INPUT_COUNT_RANGE: RangeInclusive<usize> = 1..=INPUT_COUNT_MAX; //[1..127]
/// The range of valid indices of inputs of a transaction.
pub const INPUT_INDEX_RANGE: Range<u16> = 0..INPUT_COUNT_MAX as u16; //[0..126]

/// Error encountered unpacking an [`Input`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum InputUnpackError {
    InvalidKind(u8),
    ValidationError(ValidationError),
}

impl fmt::Display for InputUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKind(kind) => write!(f, "invalid input kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A generic input supporting different input kinds.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = InputUnpackError::InvalidKind)]
#[packable(pack_error = Infallible)]
#[packable(unpack_error = MessageUnpackError)]
pub enum Input {
    /// A UTXO input.
    #[packable(tag = UtxoInput::KIND)]
    Utxo(UtxoInput),
}

impl Input {
    /// Returns the input kind of an [`Input`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Utxo(_) => UtxoInput::KIND,
        }
    }
}

impl From<UtxoInput> for Input {
    fn from(input: UtxoInput) -> Self {
        Self::Utxo(input)
    }
}

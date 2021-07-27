// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod utxo;

pub use utxo::UtxoInput;

use crate::error::{MessageUnpackError, ValidationError};

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use core::{
    convert::Infallible,
    fmt,
    ops::{Range, RangeInclusive},
};

/// The maximum number of inputs for a transaction.
pub const INPUT_COUNT_MAX: usize = 127;
/// The range of valid numbers of inputs for a transaction [1..127].
pub const INPUT_COUNT_RANGE: RangeInclusive<usize> = 1..=INPUT_COUNT_MAX;
/// The valid range of indices for inputs for a transaction [0..126].
pub const INPUT_INDEX_RANGE: Range<u16> = 0..INPUT_COUNT_MAX as u16;

/// Error encountered unpacking an input.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum InputUnpackError {
    InvalidInputKind(u8),
    ValidationError(ValidationError),
}

impl fmt::Display for InputUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInputKind(kind) => write!(f, "invalid input kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A generic input supporting different input kinds.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Input {
    /// A UTXO input.
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

impl Packable for Input {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.kind().packed_len()
            + match self {
                Self::Utxo(utxo) => utxo.packed_len(),
            }
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.kind().pack(packer).map_err(PackError::infallible)?;

        match self {
            Self::Utxo(utxo) => utxo.pack(packer).map_err(PackError::infallible)?,
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let kind = u8::unpack(unpacker).map_err(UnpackError::infallible)?;

        let variant = match kind {
            UtxoInput::KIND => Self::Utxo(UtxoInput::unpack(unpacker)?),
            tag => Err(UnpackError::Packable(InputUnpackError::InvalidInputKind(tag))).map_err(UnpackError::coerce)?,
        };

        Ok(variant)
    }
}

impl From<UtxoInput> for Input {
    fn from(input: UtxoInput) -> Self {
        Self::Utxo(input)
    }
}

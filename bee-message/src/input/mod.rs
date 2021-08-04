// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod treasury;
mod utxo;

pub use treasury::TreasuryInput;
pub use utxo::UtxoInput;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

/// A generic input supporting different input kinds.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Input {
    /// A UTXO input.
    Utxo(UtxoInput),
    /// A treasury input.
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

impl Packable for Input {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Utxo(input) => UtxoInput::KIND.packed_len() + input.packed_len(),
            Self::Treasury(input) => TreasuryInput::KIND.packed_len() + input.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Utxo(input) => {
                UtxoInput::KIND.pack(writer)?;
                input.pack(writer)?;
            }
            Self::Treasury(input) => {
                TreasuryInput::KIND.pack(writer)?;
                input.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            UtxoInput::KIND => UtxoInput::unpack_inner::<R, CHECK>(reader)?.into(),
            TreasuryInput::KIND => TreasuryInput::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidInputKind(k)),
        })
    }
}

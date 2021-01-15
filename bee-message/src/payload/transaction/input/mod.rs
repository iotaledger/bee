// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod utxo;

pub use utxo::UTXOInput;
use utxo::UTXO_INPUT_TYPE;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Hash, Ord, PartialOrd)]
#[serde(tag = "type", content = "data")]
pub enum Input {
    UTXO(UTXOInput),
}

impl From<UTXOInput> for Input {
    fn from(input: UTXOInput) -> Self {
        Self::UTXO(input)
    }
}

impl Packable for Input {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::UTXO(input) => UTXO_INPUT_TYPE.packed_len() + input.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::UTXO(input) => {
                UTXO_INPUT_TYPE.pack(writer)?;
                input.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack(reader)? {
            UTXO_INPUT_TYPE => Self::UTXO(UTXOInput::unpack(reader)?),
            _ => return Err(Self::Error::InvalidInputType),
        })
    }
}

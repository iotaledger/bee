// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod address;
mod output_id;
mod signature_locked_dust_allowance;
mod signature_locked_single;
mod storable;
mod treasury;

pub use address::{Address, Bech32Address, Ed25519Address, ED25519_ADDRESS_LENGTH};
pub use output_id::{OutputId, OUTPUT_ID_LENGTH};
pub use signature_locked_dust_allowance::SignatureLockedDustAllowanceOutput;
pub use signature_locked_single::SignatureLockedSingleOutput;
pub use storable::{ConsumedOutput, CreatedOutput};
pub use treasury::TreasuryOutput;

use signature_locked_dust_allowance::SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_KIND;
use signature_locked_single::SIGNATURE_LOCKED_SINGLE_OUTPUT_KIND;
use treasury::TREASURY_OUTPUT_KIND;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Ord, PartialOrd)]
#[serde(tag = "type", content = "data")]
pub enum Output {
    SignatureLockedSingle(SignatureLockedSingleOutput),
    SignatureLockedDustAllowance(SignatureLockedDustAllowanceOutput),
    Treasury(TreasuryOutput),
}

impl From<SignatureLockedSingleOutput> for Output {
    fn from(output: SignatureLockedSingleOutput) -> Self {
        Self::SignatureLockedSingle(output)
    }
}

impl From<SignatureLockedDustAllowanceOutput> for Output {
    fn from(output: SignatureLockedDustAllowanceOutput) -> Self {
        Self::SignatureLockedDustAllowance(output)
    }
}

impl From<TreasuryOutput> for Output {
    fn from(output: TreasuryOutput) -> Self {
        Self::Treasury(output)
    }
}

impl Output {
    pub fn kind(&self) -> u8 {
        match self {
            Self::SignatureLockedSingle(_) => SIGNATURE_LOCKED_SINGLE_OUTPUT_KIND,
            Self::SignatureLockedDustAllowance(_) => SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_KIND,
            Self::Treasury(_) => TREASURY_OUTPUT_KIND,
        }
    }
}

impl Packable for Output {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::SignatureLockedSingle(output) => {
                SIGNATURE_LOCKED_SINGLE_OUTPUT_KIND.packed_len() + output.packed_len()
            }
            Self::SignatureLockedDustAllowance(output) => {
                SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_KIND.packed_len() + output.packed_len()
            }
            Self::Treasury(output) => TREASURY_OUTPUT_KIND.packed_len() + output.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::SignatureLockedSingle(output) => {
                SIGNATURE_LOCKED_SINGLE_OUTPUT_KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::SignatureLockedDustAllowance(output) => {
                SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Treasury(output) => {
                TREASURY_OUTPUT_KIND.pack(writer)?;
                output.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack(reader)? {
            SIGNATURE_LOCKED_SINGLE_OUTPUT_KIND => {
                Self::SignatureLockedSingle(SignatureLockedSingleOutput::unpack(reader)?)
            }
            SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_KIND => {
                Self::SignatureLockedDustAllowance(SignatureLockedDustAllowanceOutput::unpack(reader)?)
            }
            TREASURY_OUTPUT_KIND => Self::Treasury(TreasuryOutput::unpack(reader)?),
            t => return Err(Self::Error::InvalidOutputKind(t)),
        })
    }
}

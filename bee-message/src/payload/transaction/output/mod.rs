// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod address;
mod output_id;
mod signature_locked_dust_allowance;
mod signature_locked_single;

pub use address::{Address, Ed25519Address, WotsAddress, ED25519_ADDRESS_LENGTH};
pub use output_id::{OutputId, OUTPUT_ID_LENGTH};
pub use signature_locked_dust_allowance::SignatureLockedDustAllowanceOutput;
pub use signature_locked_single::SignatureLockedSingleOutput;

use signature_locked_dust_allowance::SIGNATURE_LOCKED_DUST_ALLOWANCE_TYPE;
use signature_locked_single::SIGNATURE_LOCKED_SINGLE_TYPE;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Ord, PartialOrd)]
#[serde(tag = "type", content = "data")]
pub enum Output {
    SignatureLockedSingle(SignatureLockedSingleOutput),
    SignatureLockedDustAllowance(SignatureLockedDustAllowanceOutput),
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

impl Packable for Output {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::SignatureLockedSingle(output) => SIGNATURE_LOCKED_SINGLE_TYPE.packed_len() + output.packed_len(),
            Self::SignatureLockedDustAllowance(output) => {
                SIGNATURE_LOCKED_DUST_ALLOWANCE_TYPE.packed_len() + output.packed_len()
            }
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::SignatureLockedSingle(output) => {
                SIGNATURE_LOCKED_SINGLE_TYPE.pack(writer)?;
                output.pack(writer)?;
            }
            Self::SignatureLockedDustAllowance(output) => {
                SIGNATURE_LOCKED_DUST_ALLOWANCE_TYPE.pack(writer)?;
                output.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(reader)? {
            SIGNATURE_LOCKED_SINGLE_TYPE => Self::SignatureLockedSingle(SignatureLockedSingleOutput::unpack(reader)?),
            SIGNATURE_LOCKED_DUST_ALLOWANCE_TYPE => {
                Self::SignatureLockedDustAllowance(SignatureLockedDustAllowanceOutput::unpack(reader)?)
            }
            _ => return Err(Self::Error::InvalidOutputType),
        })
    }
}

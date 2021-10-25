// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod output_id;
mod signature_locked_dust_allowance;
mod signature_locked_single;
mod treasury;

pub use output_id::{OutputId, OUTPUT_ID_LENGTH};
pub use signature_locked_dust_allowance::{
    dust_outputs_max, SignatureLockedDustAllowanceOutput, DUST_ALLOWANCE_DIVISOR, DUST_OUTPUTS_MAX, DUST_THRESHOLD,
    SIGNATURE_LOCKED_DUST_ALLOWANCE_OUTPUT_AMOUNT,
};
pub use signature_locked_single::{SignatureLockedSingleOutput, SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT};
pub use treasury::{TreasuryOutput, TREASURY_OUTPUT_AMOUNT};

use crate::Error;

use bee_packable::Packable;

/// A generic output that can represent different types defining the deposit of funds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = Error::InvalidOutputKind)]
#[packable(unpack_error = Error)]
pub enum Output {
    /// A signature locked single output.
    #[packable(tag = SignatureLockedSingleOutput::KIND)]
    SignatureLockedSingle(SignatureLockedSingleOutput),
    /// A signature locked dust allowance output.
    #[packable(tag = SignatureLockedDustAllowanceOutput::KIND)]
    SignatureLockedDustAllowance(SignatureLockedDustAllowanceOutput),
    /// A treasury output.
    #[packable(tag = TreasuryOutput::KIND)]
    Treasury(TreasuryOutput),
}

impl Output {
    /// Return the output kind of an `Output`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::SignatureLockedSingle(_) => SignatureLockedSingleOutput::KIND,
            Self::SignatureLockedDustAllowance(_) => SignatureLockedDustAllowanceOutput::KIND,
            Self::Treasury(_) => TreasuryOutput::KIND,
        }
    }
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

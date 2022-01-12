// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Errors related to ledger types.
#[derive(Debug, thiserror::Error)]
pub enum ConflictError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Invalid conflict byte.
    #[error("Invalid conflict byte")]
    InvalidConflict(u8),
}

/// Represents the different reasons why a transaction can conflict with the ledger state.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, bee_packable::Packable)]
#[packable(unpack_error = ConflictError)]
#[packable(tag_type = u8, with_error = ConflictError::InvalidConflict)]
pub enum ConflictReason {
    /// The message has no conflict.
    None = 0,
    /// The referenced Utxo was already spent.
    InputUtxoAlreadySpent = 1,
    /// The referenced Utxo was already spent while confirming this milestone.
    InputUtxoAlreadySpentInThisMilestone = 2,
    /// The referenced Utxo cannot be found.
    InputUtxoNotFound = 3,
    /// The sum of the inputs and output values does not match.
    InputOutputSumMismatch = 4,
    /// The unlock block signature is invalid.
    InvalidSignature = 5,
    /// The semantic validation failed for a reason not covered by the previous variants.
    SemanticValidationFailed = 255,
}

impl Default for ConflictReason {
    fn default() -> Self {
        Self::None
    }
}

impl TryFrom<u8> for ConflictReason {
    type Error = ConflictError;

    fn try_from(c: u8) -> Result<Self, Self::Error> {
        Ok(match c {
            0 => Self::None,
            1 => Self::InputUtxoAlreadySpent,
            2 => Self::InputUtxoAlreadySpentInThisMilestone,
            3 => Self::InputUtxoNotFound,
            4 => Self::InputOutputSumMismatch,
            5 => Self::InvalidSignature,
            255 => Self::SemanticValidationFailed,
            x => return Err(Self::Error::InvalidConflict(x)),
        })
    }
}

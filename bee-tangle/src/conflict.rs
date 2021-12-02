// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;

use serde::{Deserialize, Serialize};

use std::io::{Read, Write};

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConflictReason {
    /// The message has no conflict.
    None = 0,
    /// The referenced Utxo was already spent.
    InputUtxoAlreadySpent = 1,
    /// The referenced Utxo was already spent while confirming this milestone.
    InputUtxoAlreadySpentInThisMilestone = 2,
    /// The referenced Utxo cannot be found.
    InputUtxoNotFound = 3,
    /// The created amount does not match the consumed amount.
    CreatedConsumedAmountMismatch = 4,
    /// The unlock block signature is invalid.
    InvalidSignature = 5,
    /// The dust allowance for the address is invalid.
    InvalidDustAllowance = 6,
    /// The created native tokens amount does not match the consumed native tokens amount.
    CreatedConsumedNativeTokensAmountMismatch = 7,
    /// The milestone index timelock was no satisfied.
    TimelockMilestoneIndex = 8,
    /// The unix timelock was no satisfied.
    TimelockUnix = 9,
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
            4 => Self::CreatedConsumedAmountMismatch,
            5 => Self::InvalidSignature,
            6 => Self::InvalidDustAllowance,
            7 => Self::CreatedConsumedNativeTokensAmountMismatch,
            8 => Self::TimelockMilestoneIndex,
            9 => Self::TimelockUnix,
            255 => Self::SemanticValidationFailed,

            x => return Err(Self::Error::InvalidConflict(x)),
        })
    }
}

impl Packable for ConflictReason {
    type Error = ConflictError;

    fn packed_len(&self) -> usize {
        (*self as u8).packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        Ok((*self as u8).pack(writer)?)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        u8::unpack_inner::<R, CHECK>(reader)?.try_into()
    }
}

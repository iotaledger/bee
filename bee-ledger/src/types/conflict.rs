// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    /// The dust allowance for the address is invalid.
    InvalidDustAllowance = 6,
    /// The semantic validation failed.
    SemanticValidationFailed = 255,
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConflictReason {
    /// The message has no conflict.
    None = 0,
    /// The referenced UTXO was already spent.
    InputUTXOAlreadySpent = 1,
    /// The referenced UTXO was already spent while confirming this milestone.
    InputUTXOAlreadySpentInThisMilestone = 2,
    /// The referenced UTXO cannot be found.
    InputUTXONotFound = 3,
    /// The sum of the inputs and output values does not match.
    InputOutputSumMismatch = 4,
    /// The unlock block signature is invalid.
    InvalidSignature = 5,
    /// The dust allowance for the address is invalid.
    InvalidDustAllowance = 6,
    /// The semantic validation failed.
    SemanticValidationFailed = 255,
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConflictReason {
    /// The message has no conflict.
    None,
    /// The referenced UTXO was already spent.
    InputUTXOAlreadySpent,
    /// The referenced UTXO was already spent while confirming this milestone.
    InputUTXOAlreadySpentInThisMilestone,
    /// The referenced UTXO cannot be found.
    InputUTXONotFound,
    /// The sum of the inputs and output values does not match.
    InputOutputSumMismatch,
    /// The unlock block signature is invalid.
    InvalidSignature,
    /// The input or output type used is unsupported.
    UnsupportedInputOrOutputType,
    /// The used address type is unsupported.
    UnsupportedAddressType,
    /// The dust allowance for the address is invalid.
    InvalidDustAllowance,
    /// The semantic validation failed.
    SemanticValidationFailed,
}

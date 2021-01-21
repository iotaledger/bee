// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Big integer errors.

use thiserror::Error;

/// Errors related to big integers.
#[derive(Clone, Debug, Error)]
pub enum Error {
    /// Error when converting and binary representation exceeds ternary range.
    #[error("Binary representation exceeds ternary range.")]
    BinaryExceedsTernaryRange,
    /// Error when converting and ternary representation exceeds binary range.
    #[error("Ternary representation exceeds binary range.")]
    TernaryExceedsBinaryRange,
}

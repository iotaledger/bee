// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides error types for validation and packing/unpacking.

mod packable;
mod validation;

pub use packable::{MessagePackError, MessageUnpackError};
pub use validation::ValidationError;

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::error::ValidationError as MessageError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid syntax for field \"{0}\"")]
    InvalidSyntaxField(&'static str),
    #[error("invalid semantic for field \"{0}\"")]
    InvalidSemanticField(&'static str),
    #[error("{0}")]
    Message(#[from] MessageError),
}

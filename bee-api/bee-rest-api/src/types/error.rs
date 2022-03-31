// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::Error as MessageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid field \"{0}\"")]
    InvalidField(&'static str),
    #[error("{0}")]
    Message(#[from] MessageError),
}

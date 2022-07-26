// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use inx::tonic;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("gRPC status code: {0}")]
    StatusCode(#[from] tonic::Status),
    #[error(transparent)]
    TonicError(#[from] tonic::Error),
    #[error(transparent)]
    InxError(#[from] bee_block::InxError),
}

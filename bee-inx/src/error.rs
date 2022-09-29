// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use inx::tonic;
use thiserror::Error;

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    BeeBlockError(#[from] bee_block::Error),
    #[error(transparent)]
    InxError(#[from] bee_block::InxError),
    #[error("gRPC status code: {0}")]
    StatusCode(#[from] tonic::Status),
    #[error(transparent)]
    TonicError(#[from] tonic::transport::Error),
}

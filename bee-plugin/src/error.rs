// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

use std::net::AddrParseError;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("IO error for children process: {0}")]
    Io(#[from] std::io::Error),
    #[error("gRPC transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),
    #[error("gRPC status error: {0}")]
    StatusError(#[from] tonic::Status),
    #[error("address parsing error: {0}")]
    AddressError(#[from] AddrParseError),
    #[error("invalid event ID {0}")]
    InvalidEventId(i32),
}

// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::handshake::InvalidHandshake;

use thiserror::Error;

/// Errors related to plugin management and execution.
#[derive(Error, Debug)]
pub enum PluginError {
    /// IO errors caused by the plugin child process.
    #[error("IO error for children process: {0}")]
    Io(#[from] std::io::Error),
    /// Transport errors between the node and the plugin.
    #[error("gRPC transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),
    /// Status errors from the gRPC plugin server.
    #[error("gRPC status error: {0}")]
    StatusError(#[from] tonic::Status),
    /// Invalid handshake errors.
    #[error("invalid handshake: {0}")]
    Handshake(#[from] InvalidHandshake),
}

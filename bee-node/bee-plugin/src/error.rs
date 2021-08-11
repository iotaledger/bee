// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::handshake::InvalidHandshake;

use thiserror::Error;

/// Errors related to plugin management and execution.
#[derive(Debug, Error)]
pub enum PluginError {
    /// IO errors caused by the plugin child process.
    #[error("IO error for children process: {0}")]
    Io(#[from] std::io::Error),
    /// gRPC transport error between the node and the plugin.
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    /// Status error from the gRPC plugin server.
    #[error("gRPC status error: {0}")]
    Status(#[from] tonic::Status),
    /// Invalid handshake error.
    #[error("invalid handshake error: {0}")]
    Handshake(#[from] InvalidHandshake),
}

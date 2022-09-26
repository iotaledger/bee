// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

/// Errors during network initialization.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Publishing the local (peer) id failed.
    #[error("failed to announce local id")]
    LocalIdAnnouncementFailed,

    /// Publishing the list of static peers failed.
    #[error("failed to announce static peers")]
    StaticPeersAnnouncementFailed,

    /// Creating transport layer failed.
    #[error("failed to create transport layer")]
    CreatingTransportFailed,

    /// Binding to an address failed.
    #[error("failed to bind to an address: {0}")]
    BindingAddressFailed(#[from] libp2p_core::transport::TransportError<std::io::Error>),

    /// An error occurred in the host event loop.
    #[error("failed to process an item in the host processor event loop")]
    HostEventLoopError,
}

// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

/// Errors during network initialization.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Publishing the local (peer) id failed.
    #[error("Failed to announce local id")]
    LocalIdAnnouncementFailed,

    /// Publishing the list of static peers failed.
    #[error("Failed to announce static peers.")]
    StaticPeersAnnouncementFailed,

    /// Creating transport layer failed.
    #[error("Failed to create transport layer.")]
    CreatingTransportFailed,

    /// Binding to an address failed.
    #[error("Failed to bind to an address.")]
    BindingAddressFailed,
}

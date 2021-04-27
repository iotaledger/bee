// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

/// Errors during initialization.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Publishing the local (peer) id failed.
    #[error("Failed to announce local id")]
    LocalIdAnnouncementFailed,

    /// Publishing the list of static peers failed.
    #[error("Failed to announce static peers.")]
    StaticPeersAnnouncementFailed,
}

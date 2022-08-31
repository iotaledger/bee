// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Errors that can occur during sending/receiving of [`Command`](crate::Command)s and [`Event`](crate::Event)s.
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    /// A command could not be sent.
    #[error("error sending command")]
    SendingCommandFailed,

    /// An event could not be sent.
    #[error("error sending command")]
    SendingEventFailed,

    /// An event could not been received.
    #[error("error receiving event")]
    ReceivingEventFailed,

    /// An error regarding a specific peer occurred.
    #[error("{0}")]
    PeerError(#[from] crate::peer::error::Error),
}

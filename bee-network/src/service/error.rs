// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Errors that can occur during sending/receiving of [`Command`]s and [`Event`]s.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A command could not be sent.
    #[error("Error sending command.")]
    SendingCommandFailed,

    /// An event could not be sent.
    #[error("Error sending command.")]
    SendingEventFailed,

    /// An event could not been received.
    #[error("Error receiving event.")]
    ReceivingEventFailed,

    /// An error regarding a specific peer occured.
    #[error("{:?}", 0)]
    PeerError(#[from] crate::peer::error::Error),
}

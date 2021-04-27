// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Errors that can occur during sending/receiving of [`Command`]s and [`Event`]s.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error sending command.")]
    CommandSendFailure,

    #[error("Error receiving event.")]
    EventRecvFailure,
}

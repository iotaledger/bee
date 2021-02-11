// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::PeerId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to write a message to a stream.")]
    MessageSendError,

    #[error("Failed to read a message from a stream.")]
    MessageRecvError,

    #[error("The remote peer '{}' stopped the stream (EOF).", .0)]
    StreamClosedByRemote(PeerId),
}

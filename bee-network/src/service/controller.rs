// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    service::commands::{Command, CommandSender},
    PeerId,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error sending command.")]
    CommandSendFailure,
}

/// A controller for the networking layer, that allows to issue various commands, e.g. sending a message to a peer.
#[derive(Clone, Debug)]
pub struct NetworkServiceController {
    command_sender: CommandSender,
    local_peer_id: PeerId,
}

impl NetworkServiceController {
    pub(crate) fn new(command_sender: CommandSender, local_peer_id: PeerId) -> Self {
        Self {
            command_sender,
            local_peer_id,
        }
    }

    /// Sends a command.
    /// NOTE: Although synchronous, this method never actually blocks.
    pub fn send(&self, command: Command) -> Result<(), Error> {
        self.command_sender
            .send(command)
            .map_err(|_| Error::CommandSendFailure)?;

        Ok(())
    }

    /// Returns the local peer id.
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }
}

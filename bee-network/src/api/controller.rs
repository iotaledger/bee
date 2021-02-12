// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    api::commands::{Command, CommandSender},
    PeerId,
};

// use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error sending command.")]
    CommandSendFailure,
}

/// A controller for the networking layer, that allows to issue various commands, e.g. sending a message to a peer.
#[derive(Clone, Debug)]
pub struct NetworkController {
    // config: Arc<NetworkConfig>,
    command_sender: CommandSender,
    local_peer_id: PeerId,
}

impl NetworkController {
    pub(crate) fn new(command_sender: CommandSender, local_peer_id: PeerId) -> Self {
        Self {
            // config: Arc::new(config),
            command_sender,
            local_peer_id,
        }
    }

    /// Sends a command.
    /// NOTE: Although synchronous, this method never actually blocks.
    pub fn send(&self, command: Command) -> Result<(), Error> {
        Ok(self
            .command_sender
            .send(command)
            .map_err(|_| Error::CommandSendFailure)?)
    }

    // /// Returns the network config.
    // pub fn config(&self) -> &NetworkConfig {
    //     &self.config
    // }

    /// Returns the local peer id.
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }
}

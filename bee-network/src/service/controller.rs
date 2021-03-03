// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::service::commands::{Command, CommandSender};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error sending command.")]
    CommandSendFailure,
}

/// A controller for the networking layer, that allows to issue various commands, e.g. sending a message to a peer.
#[derive(Clone, Debug)]
pub struct NetworkServiceController {
    command_sender: CommandSender,
}

impl NetworkServiceController {
    pub(crate) fn new(command_sender: CommandSender) -> Self {
        Self { command_sender }
    }

    /// Sends a command.
    /// NOTE: Although synchronous, this method never actually blocks.
    pub fn send(&self, command: Command) -> Result<(), Error> {
        self.command_sender
            .send(command)
            .map_err(|_| Error::CommandSendFailure)?;

        Ok(())
    }
}

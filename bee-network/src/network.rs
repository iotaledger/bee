// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::NetworkConfig,
    interaction::commands::{Command, CommandSender},
    PeerId,
};

use thiserror::Error;

use std::sync::Arc;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error sending command.")]
    CommandSendFailure,
}

#[derive(Clone, Debug)]
pub struct NetworkController {
    config: Arc<NetworkConfig>,
    command_sender: CommandSender,
    own_id: PeerId,
}

impl NetworkController {
    pub(crate) fn new(config: NetworkConfig, command_sender: CommandSender, own_id: PeerId) -> Self {
        Self {
            config: Arc::new(config),
            command_sender,
            own_id,
        }
    }

    /// NOTE: Although synchronous, this method never actually blocks.
    pub fn send(&self, command: Command) -> Result<(), Error> {
        Ok(self
            .command_sender
            .send(command)
            .map_err(|_| Error::CommandSendFailure)?)
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub fn own_id(&self) -> &PeerId {
        &self.own_id
    }
}

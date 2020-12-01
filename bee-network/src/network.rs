// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::NetworkConfig,
    interaction::commands::{Command, CommandSender},
    Multiaddr, PeerId,
};

use thiserror::Error;

use std::sync::Arc;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error sending command.")]
    CommandSendFailure,
    #[error("Error sending unbounded command.")]
    CommandSendUnboundedFailure,
}

#[derive(Clone, Debug)]
pub struct Network {
    config: Arc<NetworkConfig>,
    command_sender: CommandSender,
    listen_address: Multiaddr,
    local_id: PeerId,
}

impl Network {
    pub(crate) fn new(
        config: NetworkConfig,
        command_sender: CommandSender,
        listen_address: Multiaddr,
        local_id: PeerId,
    ) -> Self {
        Self {
            config: Arc::new(config),
            command_sender,
            listen_address,
            local_id,
        }
    }

    pub async fn send(&mut self, command: Command) -> Result<(), Error> {
        Ok(self
            .command_sender
            .send_async(command)
            .await
            .map_err(|_| Error::CommandSendFailure)?)
    }

    pub fn unbounded_send(&self, command: Command) -> Result<(), Error> {
        Ok(self
            .command_sender
            .send(command)
            .map_err(|_| Error::CommandSendUnboundedFailure)?)
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub fn listen_address(&self) -> &Multiaddr {
        &self.listen_address
    }

    pub fn local_id(&self) -> &PeerId {
        &self.local_id
    }
}

// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    command::{Command, CommandSender},
    event::{Event, EventReceiver},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error sending command.")]
    CommandSendFailure,
    #[error("Error receiving event.")]
    EventRecvFailure,
}

/// A controller for the networking layer, that allows to issue various commands, e.g. sending a message to a peer.
#[derive(Clone, Debug)]
pub struct NetworkCommandSender(CommandSender);

impl NetworkCommandSender {
    pub(crate) fn new(inner: CommandSender) -> Self {
        Self(inner)
    }

    /// Sends a command to the network.
    /// NOTE: Although synchronous, this method never actually blocks.
    pub fn send(&self, command: Command) -> Result<(), Error> {
        self.0.send(command).map_err(|_| Error::CommandSendFailure)?;

        Ok(())
    }
}

/// A type that receives any event published by the networking layer.
pub struct NetworkEventReceiver(EventReceiver);

impl NetworkEventReceiver {
    pub(crate) fn new(inner: EventReceiver) -> Self {
        Self(inner)
    }

    /// Waits for an event from the network.
    pub async fn recv(&mut self) -> Option<Event> {
        self.0.recv().await
    }
}

impl Into<EventReceiver> for NetworkEventReceiver {
    fn into(self) -> EventReceiver {
        self.0
    }
}

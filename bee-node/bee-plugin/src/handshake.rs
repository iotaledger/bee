// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Handshake definition for the bee plugin system.

use crate::event::{EventId, InvalidEventId};

use thiserror::Error;

use std::{
    convert::TryFrom,
    fmt::Write,
    net::{AddrParseError, SocketAddr},
};

/// Information provided by the plugin during the handshake stage.
pub struct PluginHandshake {
    pub(crate) name: String,
    pub(crate) address: SocketAddr,
    pub(crate) event_ids: Vec<EventId>,
}

impl PluginHandshake {
    /// Creates a new [`PluginHandshake`] using the plugin's name for logging purposes, the plugin's gRPC server
    /// address, and the [`EventId`]s that the plugins will be subscribed to.
    pub fn new(name: &str, address: SocketAddr, event_ids: Vec<EventId>) -> Self {
        Self {
            name: name.to_owned(),
            address,
            event_ids,
        }
    }

    pub(crate) fn parse(buf: &str) -> Result<Self, InvalidHandshake> {
        let mut chunks = buf.trim().split('|');
        let name = chunks.next().ok_or(InvalidHandshake::MissingName)?.to_string();
        let address = chunks.next().ok_or(InvalidHandshake::MissingAddress)?.parse()?;
        let event_ids = chunks
            .map(|chunk| {
                let event_id: u8 = chunk
                    .parse()
                    .map_err(|_| InvalidHandshake::InvalidEventIdType(chunk.to_owned()))?;
                Ok(EventId::try_from(event_id)?)
            })
            .collect::<Result<Vec<EventId>, InvalidHandshake>>()?;

        Ok(PluginHandshake {
            name,
            address,
            event_ids,
        })
    }

    pub(crate) fn emit(self) -> String {
        let mut buf = format!("{}|{}", self.name, self.address);

        for id in self.event_ids {
            // Writing to a string buffer cannot fail.
            write!(&mut buf, "|{}", id as u8).unwrap();
        }

        buf += "\n";

        buf
    }
}

/// Errors occurring while handshaking.
#[derive(Debug, Error)]
pub enum InvalidHandshake {
    /// The name field is missing.
    #[error("missing name field")]
    MissingName,
    /// The address field is missing.
    #[error("missing address field")]
    MissingAddress,
    /// The address field is invalid.
    #[error("invalid address field: {0}")]
    InvalidAddress(#[from] AddrParseError),
    /// Invalid event identifier.
    #[error("invalid event ID {0}")]
    InvalidEventId(u8),
    /// Invalid event identifier type.
    #[error("invalid event ID type, expected integer, found: {0}")]
    InvalidEventIdType(String),
}

impl From<InvalidEventId> for InvalidHandshake {
    fn from(InvalidEventId(id): InvalidEventId) -> Self {
        Self::InvalidEventId(id)
    }
}

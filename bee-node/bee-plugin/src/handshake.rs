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
    /// address, and the [`EventId`]s that the plugins will be suscribed to.
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
        let mut event_ids = vec![];

        for chunk in chunks {
            let raw_id: u8 = chunk
                .parse()
                .map_err(|_| InvalidHandshake::ExpectedInteger(chunk.to_owned()))?;
            let event_id = EventId::try_from(raw_id)?;
            event_ids.push(event_id);
        }

        Ok(PluginHandshake {
            name,
            address,
            event_ids,
        })
    }

    pub(crate) fn emit(self) -> String {
        let mut buf = String::new();
        // Writing to a string buffer cannot fail.
        write!(&mut buf, "{}|{}", self.address, self.name).unwrap();

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
#[allow(missing_docs)]
pub enum InvalidHandshake {
    #[error("missing address field")]
    MissingAddress,
    #[error("missing name field")]
    MissingName,
    #[error("invalid address field: {0}")]
    InvalidAddress(#[from] AddrParseError),
    #[error("invalid event ID {0}")]
    InvalidEventId(u8),
    #[error("expected integer, found: {0}")]
    ExpectedInteger(String),
}

impl From<InvalidEventId> for InvalidHandshake {
    fn from(InvalidEventId(id): InvalidEventId) -> Self {
        Self::InvalidEventId(id)
    }
}

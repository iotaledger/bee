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
pub struct HandshakeInfo {
    pub(crate) address: SocketAddr,
    pub(crate) name: String,
    pub(crate) event_ids: Vec<EventId>,
}

impl HandshakeInfo {
    /// Creates a new [`HandshakeInfo`] using the plugin's gRPC server address, the plugin's name for
    /// logging purposes and the `EventId`s that the plugins will be suscribed to.
    pub fn new(address: SocketAddr, name: &str, event_ids: Vec<EventId>) -> Self {
        Self {
            address,
            name: name.to_owned(),
            event_ids,
        }
    }

    pub(crate) fn parse(buf: &str) -> Result<Self, InvalidHandshake> {
        let mut chunks = buf.trim().split('|');
        let address_chunk = chunks.next().ok_or(InvalidHandshake::MissingAddress)?;
        let address: SocketAddr = address_chunk.parse()?;
        let name = chunks.next().ok_or(InvalidHandshake::MissingName)?.to_string();
        let mut event_ids = vec![];

        for chunk in chunks {
            let raw_id: u8 = chunk
                .parse()
                .map_err(|_| InvalidHandshake::ExpectedInteger(chunk.to_owned()))?;
            let event_id = EventId::try_from(raw_id)?;
            event_ids.push(event_id);
        }

        Ok(HandshakeInfo {
            address,
            name,
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

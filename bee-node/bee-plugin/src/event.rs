// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types for events.

pub use crate::grpc::{MessageParsedEvent, MessageRejectedEvent, ParsingFailedEvent};

use std::convert::TryFrom;

/// Identifiers for each event type.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum EventId {
    /// Identifier for [`MessageParsedEvent`].
    MessageParsed = 0,
    /// Identifier for [`ParsingFailedEvent`].
    ParsingFailed = 1,
    /// Identifier for [`MessageRejectedEvent`].
    MessageRejected = 2,
}

/// Error returned while converting an [`u8`] into an [`EventId`].
#[derive(Debug)]
pub struct InvalidEventId(pub(crate) u8);

impl TryFrom<u8> for EventId {
    type Error = InvalidEventId;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::MessageParsed),
            1 => Ok(Self::ParsingFailed),
            2 => Ok(Self::MessageRejected),
            value => Err(InvalidEventId(value)),
        }
    }
}

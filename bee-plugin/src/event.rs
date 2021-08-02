// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types for events.

pub use crate::grpc::{DummyEvent, SillyEvent};

use std::convert::TryFrom;

/// Identifier for each event type.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum EventId {
    /// Identifier for `DummyEvent`.
    Dummy = 0,
    /// Identifier for `SillyEvent`.
    Silly = 1,
}

impl TryFrom<u8> for EventId {
    type Error = InvalidEventId;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Dummy),
            1 => Ok(Self::Silly),
            value => Err(InvalidEventId(value)),
        }
    }
}

/// Error returned while converting into an `EventId`.
#[derive(Debug)]
pub struct InvalidEventId(pub(crate) u8);

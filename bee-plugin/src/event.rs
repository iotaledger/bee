// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::grpc::{DummyEvent, SillyEvent};

use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum EventId {
    Dummy = 0,
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

#[derive(Debug)]
pub struct InvalidEventId(pub u8);

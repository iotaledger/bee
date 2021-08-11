// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types for events.

pub use crate::grpc::{MessageParsedEvent, MessageRejectedEvent, ParsingFailedEvent};

use std::convert::TryFrom;

/// Error returned while converting an [`u8`] into an [`EventId`].
#[derive(Debug)]
pub struct InvalidEventId(pub(crate) u8);

macro_rules! define_event_id {
    ($($variant:ident => $int:tt),*) => {
        /// Identifier for each event type.
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        #[repr(u8)]
        pub enum EventId {
            $(
                #[doc = concat!("Identifier for [`", stringify!($variant), "Event`].")]
                $variant = $int,
            )*
        }

        impl TryFrom<u8> for EventId {
            type Error = InvalidEventId;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $($int => Ok(Self::$variant),)*
                    value => Err(InvalidEventId(value)),
                }
            }
        }
    };
}

define_event_id! {
    MessageParsed => 0,
    ParsingFailed => 1,
    MessageRejected => 2
}

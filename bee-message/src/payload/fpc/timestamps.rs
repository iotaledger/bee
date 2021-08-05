// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::PAYLOAD_LENGTH_MAX, MessageId};

use bee_packable::{error::UnpackPrefixError, BoundedU32, Packable, VecPrefix};

use alloc::vec;
use core::{
    convert::{Infallible, TryFrom},
    ops::Deref,
};

/// No [`Vec`] max length specified, so use [`PAYLOAD_LENGTH_MAX`] / length of
/// [`Conflict`](crate::payload::fpc::Conflict).
const PREFIXED_TIMESTAMPS_LENGTH_MAX: u32 =
    PAYLOAD_LENGTH_MAX / (MessageId::LENGTH + 2 * core::mem::size_of::<u8>()) as u32;

/// Provides a convenient collection of [`Timestamp`]s.
/// Describes a vote in a given round for a message timestamp.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = UnpackPrefixError<Infallible>)]
pub struct Timestamps {
    inner: VecPrefix<Timestamp, BoundedU32<0, PREFIXED_TIMESTAMPS_LENGTH_MAX>>,
}

impl Default for Timestamps {
    fn default() -> Self {
        Self {
            inner: VecPrefix::try_from(vec![]).unwrap(),
        }
    }
}

impl Deref for Timestamps {
    type Target = [Timestamp];

    fn deref(&self) -> &Self::Target {
        self.inner.as_slice()
    }
}

impl Timestamps {
    /// Creates a new [`Conflicts`](crate::payload::fpc::Conflicts) instance from a vector of
    /// [`Conflict`](crate::payload::fpc::Conflict)s.
    pub fn new(inner: VecPrefix<Timestamp, BoundedU32<0, PREFIXED_TIMESTAMPS_LENGTH_MAX>>) -> Self {
        Self { inner }
    }
}

/// Describes a vote in a given round for a message timestamp.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Timestamp {
    /// ID of the message that contains the timestamp.
    message_id: MessageId,
    /// The nodes opinion value in a given round.
    opinion: u8,
    /// Voting round number.
    round: u8,
}

impl Timestamp {
    /// Creates a new [`Timestamp`].
    pub fn new(message_id: MessageId, opinion: u8, round: u8) -> Self {
        Self {
            message_id,
            opinion,
            round,
        }
    }

    /// Returns the ID of the message that contains the timestamp.
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Returns the nodes opinion value in a given round.
    pub fn opinion(&self) -> u8 {
        self.opinion
    }

    /// Returns the voting round number.
    pub fn round(&self) -> u8 {
        self.round
    }
}

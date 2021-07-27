// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::PAYLOAD_LENGTH_MAX, MessageId, MESSAGE_ID_LENGTH};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    Packable, VecPrefix,
};

use alloc::vec::Vec;
use core::{convert::Infallible, ops::Deref};

/// No [`Vec`] max length specified, so use [`PAYLOAD_LENGTH_MAX`] / length of [`Conflict`](crate::payload::fpc::Conflict).
const PREFIXED_TIMESTAMPS_LENGTH_MAX: usize = PAYLOAD_LENGTH_MAX / (MESSAGE_ID_LENGTH + 2 * core::mem::size_of::<u8>());

/// Provides a convenient collection of [`Timestamp`]s.
/// Describes a vote in a given round for a message timestamp.
#[derive(Clone, Default, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(pack_error = PackPrefixError<Infallible, u32>)]
#[packable(unpack_error = UnpackPrefixError<Infallible, u32>)]
pub struct Timestamps {
    #[packable(wrapper = VecPrefix<Timestamp, u32, PREFIXED_TIMESTAMPS_LENGTH_MAX>)]
    inner: Vec<Timestamp>,
}

impl Deref for Timestamps {
    type Target = [Timestamp];

    fn deref(&self) -> &Self::Target {
        &self.inner.as_slice()
    }
}

impl Timestamps {
    /// Creates a new [`Conflicts`](crate::payload::fpc::Conflicts) instance from a vector of
    /// [`Conflict`](crate::payload::fpc::Conflict)s.
    pub fn new(inner: Vec<Timestamp>) -> Self {
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

    /// Returns the ID of message that contains the timestamp.
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

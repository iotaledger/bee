// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::{Packable as OldPackable, Read, Write};
use bee_message::MessageId;

use std::ops::Deref;

/// A type representing an unreferenced message.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnreferencedMessage(MessageId);

impl From<MessageId> for UnreferencedMessage {
    fn from(message_id: MessageId) -> Self {
        Self(message_id)
    }
}

impl Deref for UnreferencedMessage {
    type Target = MessageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UnreferencedMessage {
    /// Create a new `UnreferencedMessage`.
    pub fn new(message_id: MessageId) -> Self {
        message_id.into()
    }

    /// Get the message ID of this unreferenced message.
    pub fn message_id(&self) -> &MessageId {
        &self.0
    }
}

impl OldPackable for UnreferencedMessage {
    type Error = <MessageId as OldPackable>::Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(MessageId::unpack_inner::<R, CHECK>(reader)?))
    }
}

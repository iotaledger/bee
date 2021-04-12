// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::{Packable, Read, Write};
use bee_message::MessageId;

use std::ops::Deref;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnconfirmedMessage(MessageId);

impl From<MessageId> for UnconfirmedMessage {
    fn from(message_id: MessageId) -> Self {
        Self(message_id)
    }
}

impl Deref for UnconfirmedMessage {
    type Target = MessageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UnconfirmedMessage {
    pub fn new(message_id: MessageId) -> Self {
        message_id.into()
    }

    pub fn message_id(&self) -> &MessageId {
        &self.0
    }
}

impl Packable for UnconfirmedMessage {
    type Error = <MessageId as Packable>::Error;

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

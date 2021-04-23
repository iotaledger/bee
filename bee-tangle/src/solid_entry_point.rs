// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A SolidEntryPoint is a [`MessageId`] of a message that is solid even if we do not have them
//! or their past in the database. They often come from a snapshot file and allow a node to solidify
//! without needing the full tangle history.

use bee_common::packable::{Packable, Read, Write};
use bee_message::MessageId;

use ref_cast::RefCast;

use core::{convert::AsRef, ops::Deref};

/// A SolidEntryPoint is a [`MessageId`] of a message that is solid even if we do not have them
/// or their past in the database. They often come from a snapshot file and allow a node to solidify
/// without needing the full tangle history.
#[derive(RefCast)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SolidEntryPoint(MessageId);

impl SolidEntryPoint {
    /// Create a `SolidEntryPoint` from an existing `MessageId`.
    pub fn new(message_id: MessageId) -> Self {
        message_id.into()
    }

    /// Create a null `SolidEntryPoint` (the zero-message).
    pub fn null() -> Self {
        Self(MessageId::null())
    }

    /// Returns the underlying `MessageId`.
    pub fn message_id(&self) -> &MessageId {
        &self.0
    }
}

impl From<MessageId> for SolidEntryPoint {
    fn from(message_id: MessageId) -> Self {
        Self(message_id)
    }
}

impl AsRef<MessageId> for SolidEntryPoint {
    fn as_ref(&self) -> &MessageId {
        &self.0
    }
}

impl Deref for SolidEntryPoint {
    type Target = MessageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Packable for SolidEntryPoint {
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

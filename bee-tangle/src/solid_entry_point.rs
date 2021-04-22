// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The solid_entry_point module defined the [SolidEntryPoint] type which represents
//! an already solidified message in the tangle.

use bee_common::packable::{Packable, Read, Write};
use bee_message::MessageId;

use ref_cast::RefCast;

use core::ops::Deref;

/// A SolidEntryPoint is a [`MessageId`](crate::MessageId) of a message even if we do not have them
/// or their past in the database. The often come from a snapshot file and allow a node to solidify
/// without needing the full tangle history.
///
/// This is a type wrapper around a [`MessageId`](crate::MessageId) to differentiate it from a
/// non-solidified message.
///
/// Spec: #iota-protocol-rfc
/// <https://github.com/iotaledger/protocol-rfcs/blob/master/text/0005-white-flag/0005-white-flag.md>
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

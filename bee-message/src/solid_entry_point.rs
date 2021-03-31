// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::MessageId;

use bee_common::packable::{Packable, Read, Write};

use ref_cast::RefCast;

use core::ops::Deref;

#[derive(RefCast)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SolidEntryPoint(MessageId);

/// A SolidEntryPoint is a [MessageId] of a message even if we do not have them
/// or their past in the database. The often come from a snapshot file and
/// allow a node to solidify without needing the full tangle history.assert_eq!
///
/// Spec: #iota-protocol-rfc https://github.com/iotaledger/protocol-rfcs/blob/master/text/0005-white-flag/0005-white-flag.md
impl SolidEntryPoint {
    pub fn new(message_id: MessageId) -> Self {
        message_id.into()
    }

    pub fn null() -> Self {
        Self(MessageId::null())
    }

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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(MessageId::unpack(reader)?))
    }
}

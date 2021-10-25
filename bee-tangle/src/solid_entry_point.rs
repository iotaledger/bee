// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A SolidEntryPoint is a [`MessageId`](bee_message::MessageId) of a message that is solid even if we do not have them
//! or their past in the database. They often come from a snapshot file and allow a node to solidify
//! without needing the full tangle history.

use bee_message::MessageId;
use bee_packable::Packable;

use ref_cast::RefCast;

use core::{convert::AsRef, ops::Deref};

/// A SolidEntryPoint is a [`MessageId`] of a message that is solid even if we do not have them
/// or their past in the database. They often come from a snapshot file and allow a node to solidify
/// without needing the full tangle history.
#[derive(RefCast)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Packable)]
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

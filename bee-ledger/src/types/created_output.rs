// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::error::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::{output::Output, MessageId};

use core::ops::Deref;

/// Represents a newly created output.
#[derive(Clone, Debug)]
pub struct CreatedOutput {
    message_id: MessageId,
    inner: Output,
}

impl CreatedOutput {
    /// Creates a new `CreatedOutput`.
    pub fn new(message_id: MessageId, inner: Output) -> Self {
        Self { message_id, inner }
    }

    /// Returns the message id of the `CreatedOutput`.
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Returns the inner output of the `CreatedOutput`.
    pub fn inner(&self) -> &Output {
        &self.inner
    }
}

impl Deref for CreatedOutput {
    type Target = Output;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Packable for CreatedOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.message_id.packed_len() + self.inner.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.message_id.pack(writer)?;
        self.inner.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let message_id = MessageId::unpack_inner::<R, CHECK>(reader)?;
        let inner = Output::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self { message_id, inner })
    }
}

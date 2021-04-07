// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, output::Output, MessageId};

use bee_common::packable::{Packable, Read, Write};

use core::ops::Deref;

#[derive(Clone, Debug)]
pub struct CreatedOutput {
    message_id: MessageId,
    inner: Output,
}

impl CreatedOutput {
    pub fn new(message_id: MessageId, inner: Output) -> Self {
        Self { message_id, inner }
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

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

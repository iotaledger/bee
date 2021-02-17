// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, MessageId, MESSAGE_ID_LENGTH};

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use core::ops::RangeInclusive;

pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Parents {
    first: MessageId,
    others: Vec<MessageId>,
}

impl Parents {
    pub fn new(first: MessageId, others: Vec<MessageId>) -> Result<Self, Error> {
        if !MESSAGE_PARENTS_RANGE.contains(&(others.len() + 1)) {
            return Err(Error::InvalidParentsCount(others.len() + 1));
        }

        Ok(Self { first, others })
    }

    pub fn len(&self) -> usize {
        self.others.len() + 1
    }

    pub fn iter(&self) -> impl Iterator<Item = &MessageId> + '_ {
        std::iter::once(&self.first).chain(self.others.iter())
    }
}

impl Packable for Parents {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + self.len() * MESSAGE_ID_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.len() as u8).pack(writer)?;

        for parent in self.iter() {
            parent.pack(writer)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let parents_len = u8::unpack(reader)? as usize;

        if !MESSAGE_PARENTS_RANGE.contains(&parents_len) {
            return Err(Error::InvalidParentsCount(parents_len));
        }

        let first = MessageId::unpack(reader)?;

        let mut others = Vec::with_capacity(parents_len - 1);
        for _ in 0..parents_len - 1 {
            others.push(MessageId::unpack(reader)?);
        }

        Ok(Self { first, others })
    }
}

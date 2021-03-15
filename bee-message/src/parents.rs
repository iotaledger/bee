// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{utils::is_unique_sorted, Error, MessageId, MESSAGE_ID_LENGTH};

use bee_common::packable::{Packable, Read, Write};

use core::ops::{Deref, RangeInclusive};

pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parents(Vec<MessageId>);

impl Deref for Parents {
    type Target = Vec<MessageId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parents {
    pub fn new(inner: Vec<MessageId>) -> Result<Self, Error> {
        if !MESSAGE_PARENTS_RANGE.contains(&inner.len()) {
            return Err(Error::InvalidParentsCount(inner.len()));
        }

        if !is_unique_sorted(inner.iter()) {
            return Err(Error::ParentsNotUniqueSorted);
        }

        Ok(Self(inner))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &MessageId> + '_ {
        self.0.iter()
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

        let mut inner = Vec::with_capacity(parents_len);
        for _ in 0..parents_len {
            inner.push(MessageId::unpack(reader)?);
        }

        Ok(Self::new(inner)?)
    }
}

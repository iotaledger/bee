// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type ([Parents]) for storing the
//! messages directly approved by another message.

use crate::{Error, MessageId, MESSAGE_ID_LENGTH};

use bee_common::{
    ord::is_unique_sorted,
    packable::{Packable, Read, Write},
};

use core::ops::{Deref, RangeInclusive};

/// The range containing the valid number of parents (directly approved transactions).
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

/// A [`Message`](crate::Message)'s `Parents` are the [`MessageId`]s of the transactions it directly approves.
///
/// There must be between 1 and 8 parents, and they must be sorted and unique.
///
/// Spec: #iota-protocol-rfc-draft
/// <https://github.com/GalRogozinski/protocol-rfcs/blob/message/text/0017-message/0017-message.md>
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parents(Vec<MessageId>);

impl Deref for Parents {
    type Target = Vec<MessageId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// Create a new `Parents` set.
    ///
    /// The vector must contain 1-8 unique and sorted entries.
    pub fn new(inner: Vec<MessageId>) -> Result<Self, Error> {
        if !MESSAGE_PARENTS_RANGE.contains(&inner.len()) {
            return Err(Error::InvalidParentsCount(inner.len()));
        }

        if !is_unique_sorted(inner.iter().map(AsRef::as_ref)) {
            return Err(Error::ParentsNotUniqueSorted);
        }

        Ok(Self(inner))
    }

    /// Return the number of parents.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return an iterator over the parents.
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

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let parents_len = u8::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK && !MESSAGE_PARENTS_RANGE.contains(&parents_len) {
            return Err(Error::InvalidParentsCount(parents_len));
        }

        let mut inner = Vec::with_capacity(parents_len);
        for _ in 0..parents_len {
            inner.push(MessageId::unpack_inner::<R, CHECK>(reader)?);
        }

        Self::new(inner)
    }
}

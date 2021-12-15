// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{Error, MessageId};

use bee_common::{
    ord::is_unique_sorted,
    packable::{Packable, Read, Write},
};

use derive_more::Deref;

use core::ops::RangeInclusive;

/// A [`Message`](crate::Message)'s [`Parents`] are the [`MessageId`]s of the messages it directly approves.
///
/// Parents must be:
/// * in the `Parents::COUNT_RANGE` range;
/// * lexicographically sorted;
/// * unique;
#[derive(Clone, Debug, Eq, PartialEq, Deref)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[deref(forward)]
pub struct Parents(Box<[MessageId]>);

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// The range representing the valid number of parents.
    pub const COUNT_RANGE: RangeInclusive<usize> = 1..=8;

    /// Creates new [`Parents`].
    pub fn new(inner: Vec<MessageId>) -> Result<Self, Error> {
        if !Parents::COUNT_RANGE.contains(&inner.len()) {
            return Err(Error::InvalidParentsCount(inner.len()));
        }

        if !is_unique_sorted(inner.iter().map(AsRef::as_ref)) {
            return Err(Error::ParentsNotUniqueSorted);
        }

        Ok(Self(inner.into_boxed_slice()))
    }

    /// Returns the number of parents.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over the parents.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &MessageId> + '_ {
        self.0.iter()
    }
}

impl Packable for Parents {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + self.len() * MessageId::LENGTH
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

        if CHECK && !Parents::COUNT_RANGE.contains(&parents_len) {
            return Err(Error::InvalidParentsCount(parents_len));
        }

        let mut inner = Vec::with_capacity(parents_len);
        for _ in 0..parents_len {
            inner.push(MessageId::unpack_inner::<R, CHECK>(reader)?);
        }

        Self::new(inner)
    }
}

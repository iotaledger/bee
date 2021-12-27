// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{Error, MessageId};

use bee_common::{
    ord::is_unique_sorted,
    packable::{Packable as OldPackable, Read, Write},
};
use bee_packable::{
    bounded::BoundedU8,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::BoxedSlicePrefix,
    unpacker::Unpacker,
};

use derive_more::Deref;

use core::ops::RangeInclusive;

pub(crate) type ParentCount = BoundedU8<{ *Parents::COUNT_RANGE.start() }, { *Parents::COUNT_RANGE.end() }>;

/// A [`Message`](crate::Message)'s [`Parents`] are the [`MessageId`]s of the messages it directly approves.
///
/// Parents must be:
/// * in the `Parents::COUNT_RANGE` range;
/// * lexicographically sorted;
/// * unique;
#[derive(Clone, Debug, Eq, PartialEq, Deref)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[deref(forward)]
pub struct Parents(BoxedSlicePrefix<MessageId, ParentCount>);

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// The range representing the valid number of parents.
    pub const COUNT_RANGE: RangeInclusive<u8> = 1..=8;

    /// Creates new [`Parents`].
    pub fn new(inner: Vec<MessageId>) -> Result<Self, Error> {
        let inner: BoxedSlicePrefix<MessageId, ParentCount> =
            inner.into_boxed_slice().try_into().map_err(Error::InvalidParentCount)?;

        Self::from_boxed_slice::<true>(inner)
    }

    fn from_boxed_slice<const VERIFY: bool>(inner: BoxedSlicePrefix<MessageId, ParentCount>) -> Result<Self, Error> {
        if VERIFY && !is_unique_sorted(inner.iter().map(AsRef::as_ref)) {
            return Err(Error::ParentsNotUniqueSorted);
        }

        Ok(Self(inner))
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

impl bee_packable::Packable for Parents {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.0.pack(packer)
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let inner = BoxedSlicePrefix::<MessageId, ParentCount>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::InvalidParentCount(err.into_prefix().into()))?;

        Self::from_boxed_slice::<VERIFY>(inner).map_err(UnpackError::Packable)
    }
}

impl OldPackable for Parents {
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
        let parents_len = u8::unpack_inner::<R, CHECK>(reader)?;

        if CHECK && !Parents::COUNT_RANGE.contains(&parents_len) {
            return Err(Error::InvalidParentCount(
                ParentCount::try_from(usize::from(parents_len)).unwrap_err(),
            ));
        }

        let mut inner = Vec::with_capacity(parents_len.into());
        for _ in 0..parents_len {
            inner.push(MessageId::unpack_inner::<R, CHECK>(reader)?);
        }

        Self::new(inner)
    }
}

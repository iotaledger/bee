// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{Error, MessageId};

use bee_common::ord::is_unique_sorted;
use bee_packable::{
    bounded::BoundedU8,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::BoxedSlicePrefix,
    unpacker::Unpacker,
    Packable,
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

impl Packable for Parents {
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
